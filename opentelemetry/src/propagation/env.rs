//! Experimental environment-variable propagation carriers.

use crate::propagation::{Extractor, Injector};
use std::{borrow::Cow, collections::HashMap, ffi::OsStr};

/// Experimental extractor for propagated context stored in environment variables.
///
/// `EnvVarExtractor` owns caller-provided environment entries and implements
/// [`Extractor`] with the normalization rules from the OpenTelemetry
/// environment-variable carrier specification. Keys are stored as they appear
/// in the source environment, `get()` normalizes the requested propagation key
/// before lookup, and `keys()` returns only names that are already normalized.
///
/// Rust's [`Extractor`] trait returns borrowed values, so this adapter reads
/// from the environment entries supplied by the caller instead of performing
/// hidden process-environment lookups or internal caching. To adapt the
/// current process environment at child-process startup, pass
/// [`std::env::vars_os()`] to [`EnvVarExtractor::from_os_entries`] at the
/// extraction point.
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

    /// Builds an extractor from the provided UTF-8 environment entries.
    ///
    /// Entries are stored exactly as provided. `get()` still reads only the
    /// normalized form of a propagation key, and `keys()` still returns only
    /// already-normalized names.
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
    /// Any entry whose name or value is not valid UTF-8 is ignored. This is
    /// useful when passing [`std::env::vars_os()`] explicitly at the extraction
    /// point.
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
/// default. If you need an explicit copy of existing environment entries, pass
/// them to [`EnvVarInjector::from_entries`] or
/// [`EnvVarInjector::from_os_entries`].
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

    /// Builds an injector from the provided UTF-8 environment entries.
    ///
    /// Existing entries are stored exactly as provided. Calls to [`Injector::set`]
    /// normalize only the propagation keys added through this injector API.
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

    /// Consumes the injector and returns the underlying environment-variable map.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.env
    }

    /// Builds an injector from OS-string environment entries.
    ///
    /// Any entry whose name or value is not valid UTF-8 is ignored.
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

impl<'a> IntoIterator for &'a EnvVarInjector {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.env.iter()
    }
}

fn collect_entries<I, K, V>(iter: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    iter.into_iter()
        .map(|(key, value)| (key.into(), value.into()))
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
            Some((
                key.as_ref().to_str()?.to_string(),
                value.as_ref().to_str()?.to_string(),
            ))
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
    use std::collections::HashSet;
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
                assert!(
                    Extractor::keys(&extractor).contains(&"OTEL_ENV_VAR_EXTRACTOR_TEST"),
                    "normalized keys should be visible"
                );
                assert!(
                    !Extractor::keys(&extractor).contains(&"otel.env.var.extractor.other"),
                    "non-normalized keys should be hidden"
                );
            },
        );
    }

    #[test]
    fn injector_normalizes_inserted_names() {
        let mut injector = EnvVarInjector::from_entries([("PATH", "/bin"), ("traceparent", "old")]);
        Injector::reserve(&mut injector, 2);
        Injector::set(&mut injector, "x-b3-traceid", "trace-id".to_string());
        Injector::set(&mut injector, "3trace", "prefixed".to_string());

        let env = injector.into_inner();

        assert_eq!(env.get("PATH").map(String::as_str), Some("/bin"));
        assert_eq!(env.get("traceparent").map(String::as_str), Some("old"));
        assert_eq!(
            env.get("X_B3_TRACEID").map(String::as_str),
            Some("trace-id")
        );
        assert_eq!(env.get("_3TRACE").map(String::as_str), Some("prefixed"));
    }

    #[test]
    fn injector_from_os_entries_preserves_utf8_entries() {
        with_vars(vec![("OTEL_ENV_VAR_INJECTOR_TEST", Some("value"))], || {
            let env = EnvVarInjector::from_os_entries(std::env::vars_os()).into_inner();

            assert_eq!(
                env.get("OTEL_ENV_VAR_INJECTOR_TEST").map(String::as_str),
                Some("value")
            );
        });
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

        let env = injector.into_inner();
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
