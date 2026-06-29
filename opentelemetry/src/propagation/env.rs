//! Experimental environment-variable propagation carriers.

use crate::propagation::{Extractor, Injector};
use std::{borrow::Cow, collections::HashMap, ffi::OsStr};

/// Experimental extractor for propagated context stored in environment variables.
///
/// `EnvVarExtractor` owns caller-provided environment entries and implements
/// [`Extractor`] with the normalization rules from the OpenTelemetry
/// environment-variable carrier specification. Only keys that are already
/// normalized are retained, `get()` normalizes the requested propagation key
/// before lookup, and `keys()` returns the retained names.
///
/// The extractor stores environment values because [`Extractor::get`] must
/// return `&str`, while [`std::env::var_os`] returns owned values. Storing values
/// gives the carrier stable owned storage to borrow from and makes
/// [`Extractor::keys`] operate over a consistent view instead of repeatedly
/// reading process-global state.
///
/// Most callers should pass the active propagator's fields to
/// [`EnvVarExtractor::from_fields`] at child-process startup so only known
/// propagation variables are read from the environment. Use
/// [`EnvVarExtractor::from_os_entries`] when a propagator needs
/// [`Extractor::keys`] to see the whole environment, such as legacy propagators
/// that scan carrier keys by prefix.
///
/// Environment variables are visible to other code running in the process and
/// may be visible to other users or processes with sufficient permissions, so
/// they are not suitable for sensitive data.
#[cfg_attr(docsrs, doc(cfg(feature = "otel_unstable")))]
#[derive(Clone, Debug, Default)]
pub struct EnvVarExtractor {
    env: HashMap<String, String>,
}

impl EnvVarExtractor {
    /// Creates an empty environment-variable extractor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds an extractor by reading the normalized form of each provided field
    /// from the current process environment.
    ///
    /// This is the recommended constructor when extracting with a known
    /// [`crate::propagation::TextMapPropagator`]. It avoids enumerating the whole
    /// environment for propagators that only call [`Extractor::get`] for their
    /// advertised fields. Any value that is not valid UTF-8 is ignored.
    pub fn from_fields<'a, I>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self {
            env: collect_fields(fields),
        }
    }

    /// Builds an extractor from the provided UTF-8 environment entries.
    ///
    /// Only entries whose names are already normalized are stored. `get()` still
    /// reads only the normalized form of a propagation key, and `keys()` returns
    /// the retained names.
    ///
    /// Lookup in this snapshot is case-sensitive on all platforms. This can
    /// differ from [`EnvVarExtractor::from_fields`] on platforms such as Windows,
    /// where reading a normalized name from the process environment may match an
    /// environment variable whose name differs only by case.
    pub fn from_entries<I, K, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            env: collect_entries(iter),
        }
    }

    /// Builds an extractor from OS-string environment entries.
    ///
    /// Any entry whose name or value is not valid UTF-8 is ignored. Entries
    /// whose names are not already normalized are also ignored. This scans the
    /// provided entries, so prefer [`EnvVarExtractor::from_fields`] unless a
    /// propagator needs [`Extractor::keys`] to see the whole environment.
    ///
    /// Lookup in this snapshot is case-sensitive on all platforms. This can
    /// differ from [`EnvVarExtractor::from_fields`] on platforms such as Windows,
    /// where reading a normalized name from the process environment may match an
    /// environment variable whose name differs only by case.
    pub fn from_os_entries<I, K, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        Self {
            env: collect_os_entries(iter),
        }
    }
}

impl Extractor for EnvVarExtractor {
    fn get(&self, key: &str) -> Option<&str> {
        let normalized = normalize_env_var_key(key);
        self.env.get(normalized.as_ref()).map(String::as_str)
    }

    fn keys(&self) -> Vec<&str> {
        self.env
            .keys()
            .filter(|key| is_normalized_env_var_name(key))
            .map(String::as_str)
            .collect()
    }
}

/// Experimental injector for child-process environment-variable propagation.
///
/// `EnvVarInjector` owns environment entries and implements [`Injector`] by
/// normalizing each propagation key before storing it. This makes it suitable
/// for passing to process-spawning APIs such as
/// [`std::process::Command::envs`], while leaving the parent process
/// environment untouched.
///
/// Most callers can start with [`EnvVarInjector::new`] and let
/// [`std::process::Command`] inherit the rest of the parent environment by
/// default.
#[cfg_attr(docsrs, doc(cfg(feature = "otel_unstable")))]
#[derive(Clone, Debug, Default)]
pub struct EnvVarInjector {
    env: HashMap<String, String>,
}

impl EnvVarInjector {
    /// Creates an empty environment-variable injector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Injector for EnvVarInjector {
    fn set(&mut self, key: &str, value: String) {
        self.env
            .insert(normalize_env_var_key(key).into_owned(), value);
    }

    fn reserve(&mut self, additional: usize) {
        self.env.reserve(additional);
    }
}

impl IntoIterator for EnvVarInjector {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.env.into_iter()
    }
}

fn collect_entries<I, K, V>(iter: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    iter.into_iter()
        .filter_map(|(key, value)| {
            let key = key.into();
            is_normalized_env_var_name(&key).then(|| (key, value.into()))
        })
        .collect()
}

fn collect_fields<'a, I>(fields: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = &'a str>,
{
    fields
        .into_iter()
        .filter_map(|field| {
            let normalized = normalize_env_var_key(field);
            std::env::var_os(normalized.as_ref()).and_then(|value| {
                value
                    .into_string()
                    .ok()
                    .map(|value| (normalized.into_owned(), value))
            })
        })
        .collect()
}

fn collect_os_entries<I, K, V>(iter: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    iter.into_iter()
        .filter_map(|(key, value)| {
            let key = key.as_ref().to_str()?;
            if is_normalized_env_var_name(key) {
                Some((key.to_string(), value.as_ref().to_str()?.to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn normalize_env_var_key(key: &str) -> Cow<'_, str> {
    if is_normalized_env_var_name(key) {
        return Cow::Borrowed(key);
    }

    if key.is_empty() {
        return Cow::Borrowed("_");
    }

    let mut normalized = String::with_capacity(
        key.len() + usize::from(key.as_bytes().first().is_some_and(u8::is_ascii_digit)),
    );

    if key.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        normalized.push('_');
    }

    for ch in key.chars() {
        normalized.push(match ch {
            'a'..='z' => ch.to_ascii_uppercase(),
            'A'..='Z' | '0'..='9' | '_' => ch,
            _ => '_',
        });
    }

    Cow::Owned(normalized)
}

fn is_normalized_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_uppercase() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propagation::{Extractor, Injector};
    use std::collections::{HashMap, HashSet};
    use temp_env::with_vars;

    #[test]
    fn normalize_env_var_key_matches_spec() {
        assert_eq!(normalize_env_var_key("traceparent"), "TRACEPARENT");
        assert_eq!(normalize_env_var_key("tracestate"), "TRACESTATE");
        assert_eq!(normalize_env_var_key("x-b3-traceid"), "X_B3_TRACEID");
        assert_eq!(normalize_env_var_key("3trace"), "_3TRACE");
        assert_eq!(normalize_env_var_key(""), "_");
        assert_eq!(normalize_env_var_key("héllo.world"), "H_LLO_WORLD");
    }

    #[test]
    fn normalized_env_var_detection_matches_spec() {
        assert!(is_normalized_env_var_name("TRACEPARENT"));
        assert!(is_normalized_env_var_name("_3TRACE"));
        assert!(is_normalized_env_var_name("TRACE_STATE_2"));
        assert!(!is_normalized_env_var_name(""));
        assert!(!is_normalized_env_var_name("traceparent"));
        assert!(!is_normalized_env_var_name("3TRACE"));
        assert!(!is_normalized_env_var_name("TRACE-STATE"));
    }

    #[test]
    fn extractor_reads_only_normalized_names() {
        let extractor = EnvVarExtractor::from_entries([
            ("TRACEPARENT", "normalized"),
            ("traceparent", "ignored"),
            ("x-b3-traceid", "ignored"),
            ("X_B3_TRACEID", "normalized-b3"),
        ]);

        assert_eq!(
            Extractor::get(&extractor, "traceparent"),
            Some("normalized")
        );
        assert_eq!(
            Extractor::get(&extractor, "x-b3-traceid"),
            Some("normalized-b3")
        );
    }

    #[test]
    fn extractor_from_fields_reads_only_requested_environment_names() {
        with_vars(
            vec![
                ("TRACEPARENT", Some("normalized")),
                ("TRACESTATE", Some("state")),
                ("BAGGAGE", Some("not-requested")),
            ],
            || {
                let extractor =
                    EnvVarExtractor::from_fields(["traceparent", "tracestate", "missing"]);

                assert_eq!(
                    Extractor::get(&extractor, "traceparent"),
                    Some("normalized")
                );
                assert_eq!(Extractor::get(&extractor, "tracestate"), Some("state"));
                assert_eq!(Extractor::get(&extractor, "baggage"), None);

                let keys = Extractor::keys(&extractor)
                    .into_iter()
                    .collect::<HashSet<_>>();
                assert_eq!(keys, HashSet::from(["TRACEPARENT", "TRACESTATE"]));
            },
        );
    }

    #[test]
    fn extractor_keys_return_only_normalized_names() {
        let extractor = EnvVarExtractor::from_entries([
            ("TRACEPARENT", "value"),
            ("traceparent", "ignored"),
            ("TRACESTATE", "value"),
            ("baggage", "ignored"),
        ]);

        let keys = Extractor::keys(&extractor)
            .into_iter()
            .collect::<HashSet<_>>();

        assert_eq!(keys, HashSet::from(["TRACEPARENT", "TRACESTATE"]));
    }

    #[test]
    fn extractor_from_os_entries_ignores_non_normalized_entries() {
        with_vars(
            vec![
                ("OTEL_ENV_VAR_EXTRACTOR_TEST", Some("value")),
                ("otel.env.var.extractor.other", Some("ignored")),
            ],
            || {
                let extractor = EnvVarExtractor::from_os_entries(std::env::vars_os());

                assert_eq!(
                    Extractor::get(&extractor, "OTEL_ENV_VAR_EXTRACTOR_TEST"),
                    Some("value")
                );
                assert_eq!(
                    Extractor::get(&extractor, "otel.env.var.extractor.other"),
                    None
                );

                let keys = Extractor::keys(&extractor)
                    .into_iter()
                    .collect::<HashSet<_>>();
                assert!(
                    keys.contains("OTEL_ENV_VAR_EXTRACTOR_TEST"),
                    "normalized keys should be visible"
                );
                assert!(
                    !keys.contains("otel.env.var.extractor.other"),
                    "non-normalized keys should be hidden"
                );
            },
        );
    }

    #[test]
    fn injector_normalizes_inserted_names() {
        let mut injector = EnvVarInjector::new();
        Injector::reserve(&mut injector, 2);
        Injector::set(&mut injector, "x-b3-traceid", "trace-id".to_string());
        Injector::set(&mut injector, "3trace", "prefixed".to_string());

        let env: HashMap<_, _> = injector.into_iter().collect();

        assert_eq!(
            env.get("X_B3_TRACEID").map(String::as_str),
            Some("trace-id")
        );
        assert_eq!(env.get("_3TRACE").map(String::as_str), Some("prefixed"));
    }

    #[test]
    fn injector_and_extractor_round_trip_propagation_keys() {
        let mut injector = EnvVarInjector::new();

        Injector::set(
            &mut injector,
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string(),
        );
        Injector::set(&mut injector, "tracestate", "foo=bar".to_string());

        let env: HashMap<_, _> = injector.into_iter().collect();
        assert!(env.contains_key("TRACEPARENT"));
        assert!(env.contains_key("TRACESTATE"));

        let extractor = EnvVarExtractor::from_entries(env);

        assert_eq!(
            Extractor::get(&extractor, "traceparent"),
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
        );
        assert_eq!(Extractor::get(&extractor, "tracestate"), Some("foo=bar"));
    }
}
