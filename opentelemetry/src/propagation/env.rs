//! # Environment variable carriers
//!
//! Propagates context across process boundaries through environment variables.
//! See the [environment carrier specification].
//!
//! [`EnvExtractor`] reads the current process environment and [`EnvInjector`]
//! writes to a map that is passed on to a child process. Both work with any
//! [`TextMapPropagator`] and treat values as opaque strings. They normalize keys
//! with [`normalize_key`], so `traceparent` becomes `TRACEPARENT` and
//! `x-b3-traceid` becomes `X_B3_TRACEID`.
//!
//! This module never spawns a process and never modifies the environment of the
//! current process.
//!
//! # Extracting at startup
//!
//! Treat context variables as input to process startup. Extract once during
//! initialization and keep the resulting [`Context`]. [`EnvExtractor`] reads the
//! environment on every call and doesn't cache anything, so later changes to the
//! environment are visible to later extractions.
//!
//! ```
//! use opentelemetry::propagation::{env::EnvExtractor, TextMapPropagator};
//! use opentelemetry::Context;
//!
//! fn context_from_parent(propagator: &dyn TextMapPropagator) -> Context {
//!     propagator.extract(&EnvExtractor)
//! }
//! ```
//!
//! # Passing context to a child process
//!
//! Use [`inject_into_command`] with [`std::process::Command`] and use a separate
//! command for each child so that their contexts stay isolated.
//!
//! ```
//! use opentelemetry::propagation::{env, TextMapPropagator};
//! use opentelemetry::Context;
//! use std::process::Command;
//!
//! fn child(propagator: &dyn TextMapPropagator, cx: &Context) -> Command {
//!     let mut command = Command::new("my-tool");
//!     env::inject_into_command(propagator, cx, &mut command);
//!     command
//! }
//! ```
//!
//! [`EnvInjector`] also supports directly injecting into a map for cases where
//! [`inject_into_command`] is not appropriate. Note that [`std::process::Command`]
//! automatically inherits the current process's environment, so it is important
//! to manually unset any of the propagator's fields that may have been injected
//! into the current environment by the parent process. This logic is already
//! handled by [`inject_into_command`].
//!
//! ```
//! use opentelemetry::propagation::{
//!     env::{self, EnvInjector},
//!     TextMapPropagator,
//! };
//! use opentelemetry::Context;
//! use std::collections::HashMap;
//! use std::process::Command;
//!
//! fn configure(propagator: &dyn TextMapPropagator, cx: &Context, command: &mut Command) {
//!     let mut vars = HashMap::new();
//!     propagator.inject_context(cx, &mut EnvInjector(&mut vars));
//!     for field in propagator.fields() {
//!         command.env_remove(env::normalize_key(field));
//!     }
//!     command.envs(&vars);
//! }
//! ```

use super::{Extractor, Injector, TextMapPropagator};
use crate::Context;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::process::Command;

/// Converts a propagation key into a normalized environment variable name.
///
/// An empty key becomes `_`, ASCII letters are uppercased, every other character
/// that isn't an ASCII digit or `_` becomes `_` and a name that would start with
/// an ASCII digit is prefixed with `_`.
pub fn normalize_key(key: &str) -> String {
    if key.is_empty() {
        return "_".to_owned();
    }

    let mut normalized = String::with_capacity(key.len() + 1);
    if key.starts_with(|c: char| c.is_ascii_digit()) {
        normalized.push('_');
    }
    normalized.extend(key.chars().map(|c| {
        if c.is_ascii_alphanumeric() || c == '_' {
            c.to_ascii_uppercase()
        } else {
            '_'
        }
    }));
    normalized
}

/// Whether `name` matches `^[A-Z_][A-Z0-9_]*$`.
fn is_normalized(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Reads propagation fields from the environment of the current process.
#[derive(Clone, Copy, Debug, Default)]
pub struct EnvExtractor;

impl Extractor for EnvExtractor {
    /// Reads the variable named by the normalized form of `key`. Returns `None` if
    /// it isn't set or its value isn't valid Unicode.
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        env::var(normalize_key(key)).ok().map(Cow::Owned)
    }

    /// Collects the names of all variables whose names are already normalized.
    fn keys(&self) -> Vec<Cow<'_, str>> {
        env::vars_os()
            .filter_map(|(name, _)| name.into_string().ok())
            .filter(|name| is_normalized(name))
            .map(Cow::Owned)
            .collect()
    }
}

/// Writes propagation fields to a map of environment variables for a child process.
#[derive(Debug)]
pub struct EnvInjector<'a>(pub &'a mut HashMap<String, String>);

impl Injector for EnvInjector<'_> {
    /// Sets the value under the normalized form of `key`.
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(normalize_key(key), value);
    }

    /// Reserves capacity for at least `additional` more entries to be inserted.
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

/// Configures `command` to pass `cx` to the child process it spawns.
///
/// Removes the normalized form of each of the propagator's fields from the
/// child's environment, then sets the fields that the propagator injects, so the
/// child can't inherit stale values from this process. Other variables are left
/// as they are.
pub fn inject_into_command(
    propagator: &dyn TextMapPropagator,
    cx: &Context,
    command: &mut Command,
) {
    let mut vars = HashMap::new();
    propagator.inject_context(cx, &mut EnvInjector(&mut vars));
    for field in propagator.fields() {
        command.env_remove(normalize_key(field));
    }
    command.envs(&vars);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propagation::text_map_propagator::FieldIter;
    use std::ffi::{OsStr, OsString};

    #[test]
    fn normalize_key_follows_the_specification() {
        for (key, expected) in [
            ("", "_"),
            ("traceparent", "TRACEPARENT"),
            ("x-b3-traceid", "X_B3_TRACEID"),
            ("1abc", "_1ABC"),
            ("é", "_"),
            ("aéb", "A_B"),
            ("a.b c", "A_B_C"),
            ("_", "_"),
            ("ALREADY_NORMALIZED_1", "ALREADY_NORMALIZED_1"),
        ] {
            assert_eq!(normalize_key(key), expected, "key {key:?}");
            assert!(is_normalized(expected), "expected {expected:?}");
            assert_eq!(normalize_key(expected), expected, "idempotent {expected:?}");
        }
    }

    #[test]
    fn is_normalized_matches_the_specification() {
        for name in ["_", "A", "A1", "_1", "A_B"] {
            assert!(is_normalized(name), "{name:?}");
        }
        for name in ["", "1", "1A", "a", "Aa", "A-B", "A B", "É"] {
            assert!(!is_normalized(name), "{name:?}");
        }
    }

    #[test]
    fn get_reads_the_normalized_name() {
        temp_env::with_var("ENV_CARRIER_TEST_GET", Some("value"), || {
            assert_eq!(
                EnvExtractor.get("env-carrier-test-get").as_deref(),
                Some("value")
            );
        });
    }

    #[test]
    fn get_returns_none_when_unset() {
        temp_env::with_var_unset("ENV_CARRIER_TEST_UNSET", || {
            assert_eq!(EnvExtractor.get("env-carrier-test-unset"), None);
        });
    }

    // Lookup is case-insensitive on Windows.
    #[cfg(unix)]
    #[test]
    fn get_does_not_read_a_non_normalized_name() {
        temp_env::with_vars(
            [
                ("env-carrier-test-dashed", Some("value")),
                ("env_carrier_test_lower", Some("value")),
            ],
            || {
                assert_eq!(EnvExtractor.get("env-carrier-test-dashed"), None);
                assert_eq!(EnvExtractor.get("env_carrier_test_lower"), None);
            },
        );
    }

    #[test]
    fn get_reads_the_live_environment() {
        temp_env::with_var("ENV_CARRIER_TEST_LIVE", Some("first"), || {
            let extractor = EnvExtractor;
            assert_eq!(
                extractor.get("ENV_CARRIER_TEST_LIVE").as_deref(),
                Some("first")
            );
            temp_env::with_var("ENV_CARRIER_TEST_LIVE", Some("second"), || {
                assert_eq!(
                    extractor.get("ENV_CARRIER_TEST_LIVE").as_deref(),
                    Some("second")
                );
            });
        });
    }

    #[test]
    fn values_are_opaque() {
        for value in ["", "  spaced  ", "not-a-traceparent", "a=b,c;d"] {
            temp_env::with_var("ENV_CARRIER_TEST_OPAQUE", Some(value), || {
                assert_eq!(
                    EnvExtractor.get("ENV_CARRIER_TEST_OPAQUE").as_deref(),
                    Some(value)
                );
            });

            let mut vars = HashMap::new();
            EnvInjector(&mut vars).set("env-carrier-test-opaque", value.to_owned());
            assert_eq!(
                vars.get("ENV_CARRIER_TEST_OPAQUE").map(String::as_str),
                Some(value)
            );
        }
    }

    #[test]
    fn keys_returns_only_normalized_names() {
        temp_env::with_vars(
            [
                ("ENV_CARRIER_TEST_KEYS", Some("value")),
                ("env_carrier_test_keys_lower", Some("value")),
                ("ENV-CARRIER-TEST-KEYS-DASHED", Some("value")),
            ],
            || {
                let keys = EnvExtractor.keys();
                assert!(keys.iter().all(|key| is_normalized(key)));
                assert!(keys.contains(&Cow::Borrowed("ENV_CARRIER_TEST_KEYS")));
            },
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_entries_do_not_panic() {
        use std::os::unix::ffi::OsStrExt;

        let not_unicode = OsStr::from_bytes(b"value\xff");
        temp_env::with_vars(
            [
                (OsStr::new("ENV_CARRIER_TEST_BAD_VALUE"), Some(not_unicode)),
                (
                    OsStr::from_bytes(b"ENV_CARRIER_TEST_BAD_NAME\xff"),
                    Some(OsStr::new("value")),
                ),
            ],
            || {
                assert_eq!(EnvExtractor.get("ENV_CARRIER_TEST_BAD_VALUE"), None);
                let keys = EnvExtractor.keys();
                assert!(keys.contains(&Cow::Borrowed("ENV_CARRIER_TEST_BAD_VALUE")));
            },
        );
    }

    #[test]
    fn injector_normalizes_keys() {
        let mut vars = HashMap::new();
        let mut injector = EnvInjector(&mut vars);
        injector.reserve(2);
        injector.set("x-b3-traceid", "value".to_owned());
        injector.set("", "empty".to_owned());

        assert_eq!(vars.len(), 2);
        assert_eq!(vars.get("X_B3_TRACEID").map(String::as_str), Some("value"));
        assert_eq!(vars.get("_").map(String::as_str), Some("empty"));
    }

    /// Declares two fields and only ever injects the first.
    #[derive(Debug)]
    struct PartialPropagator(Vec<String>);

    impl PartialPropagator {
        fn new() -> Self {
            PartialPropagator(vec!["first-field".to_owned(), "second-field".to_owned()])
        }
    }

    impl TextMapPropagator for PartialPropagator {
        fn inject_context(&self, _cx: &Context, injector: &mut dyn Injector) {
            injector.set("first-field", "new".to_owned());
        }

        fn extract_with_context(&self, cx: &Context, _extractor: &dyn Extractor) -> Context {
            cx.clone()
        }

        fn fields(&self) -> FieldIter<'_> {
            FieldIter::new(&self.0)
        }
    }

    #[test]
    fn inject_into_command_replaces_only_the_propagator_fields() {
        let mut command = Command::new("unused");
        command.env("FIRST_FIELD", "stale");
        command.env("SECOND_FIELD", "stale");
        command.env("UNRELATED", "kept");

        inject_into_command(&PartialPropagator::new(), &Context::new(), &mut command);

        let envs: HashMap<OsString, Option<OsString>> = command
            .get_envs()
            .map(|(name, value)| (name.to_owned(), value.map(OsStr::to_owned)))
            .collect();
        assert_eq!(envs[OsStr::new("FIRST_FIELD")], Some("new".into()));
        assert_eq!(envs[OsStr::new("SECOND_FIELD")], None);
        assert_eq!(envs[OsStr::new("UNRELATED")], Some("kept".into()));
    }
}
