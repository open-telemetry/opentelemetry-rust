//! Environment variable carriers for context propagation.
//!
//! These carriers implement the [environment variable context propagation
//! specification], which defines how propagators exchange context across
//! process boundaries -- for example, when a parent process spawns a child
//! process for batch processing, CI/CD, or command-line tooling, where network
//! protocols are not applicable.
//!
//! The carriers are format-agnostic: they treat values as opaque strings and
//! only normalize *key* names into valid POSIX environment variable names.
//! Choosing key names and encoding/decoding values remains the responsibility
//! of the [`TextMapPropagator`] driving them.
//!
//! - [`EnvironmentExtractor`] is an [`Extractor`] over a snapshot of the current
//!   process environment. A child process constructs one at startup and hands
//!   it to `TextMapPropagator::extract` to recover the parent's context.
//! - [`EnvironmentInjector`] is an [`Injector`] that collects the key/value
//!   pairs a propagator writes into a [`HashMap`], ready to pass to the
//!   environment of a child process (e.g. [`std::process::Command::envs`]).
//!   Per the spec, applications should not mutate their own context-related
//!   environment variables, so this type never touches the current process
//!   environment.
//!
//! [environment variable context propagation specification]: https://opentelemetry.io/docs/specs/otel/context/env-carriers/
//! [`TextMapPropagator`]: crate::propagation::TextMapPropagator

use std::collections::HashMap;

use super::{Extractor, Injector};

/// Normalize `key` into a valid POSIX environment variable name, following the
/// [key name normalization] rules:
///
/// - An empty name becomes `_`.
/// - `A`-`Z`, `0`-`9`, and `_` are kept as-is.
/// - `a`-`z` are uppercased.
/// - Every other character (including non-ASCII) is replaced with `_`.
/// - If the result would start with a digit, an `_` is prepended.
///
/// The result always matches the regex `^[A-Z_][A-Z0-9_]*$`. For example,
/// `x-b3-traceid` normalizes to `X_B3_TRACEID`.
///
/// [key name normalization]: https://opentelemetry.io/docs/specs/otel/context/env-carriers/#key-name-normalization
fn normalize_key(key: &str) -> String {
    if key.is_empty() {
        return String::from("_");
    }

    // At most one extra byte, for a leading `_` when the name starts with a
    // digit. char_indices/chars keeps this per-Unicode-scalar-value, matching
    // the Go and Python reference implementations.
    let mut result = String::with_capacity(key.len() + 1);
    for c in key.chars() {
        match c {
            'A'..='Z' | '0'..='9' | '_' => result.push(c),
            'a'..='z' => result.push(c.to_ascii_uppercase()),
            _ => result.push('_'),
        }
    }

    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, '_');
    }

    result
}

/// Whether `c` is valid within an already-normalized key.
fn is_normalized_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | '0'..='9' | '_')
}

/// Whether `key` is already a non-empty normalized environment variable name,
/// i.e. [`normalize_key`] would return it unchanged.
fn is_normalized_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        // Normalized names cannot be empty or start with a digit.
        Some(first) if !first.is_ascii_digit() && is_normalized_char(first) => {}
        _ => return false,
    }
    chars.all(is_normalized_char)
}

/// An [`Extractor`] that reads propagation context from environment variables.
///
/// The snapshot is taken once, at construction time, and only retains variables
/// whose names are already normalized -- per the spec, the environment is
/// treated as immutable within a process. In environments where a
/// context-carrying variable changes between logical requests (e.g. AWS
/// Lambda's `_X_AMZN_TRACE_ID`), build a fresh extractor at the start of each
/// request.
///
/// ```
/// use opentelemetry::propagation::{Extractor, environment::EnvironmentExtractor};
///
/// let extractor = EnvironmentExtractor::new();
/// // Hand `extractor` to `TextMapPropagator::extract`, or read from it directly.
/// // Lookup keys are normalized, so `traceparent` reads `TRACEPARENT`.
/// let _ = extractor.get("traceparent");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvironmentExtractor {
    vars: HashMap<String, String>,
}

impl EnvironmentExtractor {
    /// Create an extractor from a snapshot of the current process environment,
    /// keeping only variables whose names are already normalized.
    pub fn new() -> Self {
        std::env::vars().collect()
    }
}

impl FromIterator<(String, String)> for EnvironmentExtractor {
    /// Build an extractor from arbitrary key/value pairs, keeping only those
    /// whose keys are already normalized. This mirrors [`EnvironmentExtractor::new`]
    /// and is primarily useful for testing without mutating the real environment.
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self {
            vars: iter
                .into_iter()
                .filter(|(k, _)| is_normalized_key(k))
                .collect(),
        }
    }
}

impl Extractor for EnvironmentExtractor {
    /// Get the value for `key`, normalizing `key` before lookup.
    fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(&normalize_key(key)).map(String::as_str)
    }

    /// Collect all (already-normalized) keys in the snapshot.
    fn keys(&self) -> Vec<&str> {
        self.vars.keys().map(String::as_str).collect()
    }
}

/// An [`Injector`] that collects context into a set of environment variables.
///
/// Keys are normalized before insertion. The resulting map is intended to be
/// passed to the environment of a *child* process (for example via
/// [`std::process::Command::envs`]); this type deliberately never mutates the
/// current process environment.
///
/// ```
/// use opentelemetry::propagation::{Injector, environment::EnvironmentInjector};
///
/// let mut injector = EnvironmentInjector::new();
/// // Hand `injector` to `TextMapPropagator::inject_context`, or write directly.
/// injector.set("traceparent", "00-...-...-01".to_string());
///
/// let env = injector.into_vars(); // -> HashMap { "TRACEPARENT": "00-...-...-01" }
/// // std::process::Command::new("child").envs(&env)...
/// # let _ = env;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvironmentInjector {
    vars: HashMap<String, String>,
}

impl EnvironmentInjector {
    /// Create an empty injector.
    pub fn new() -> Self {
        Self::default()
    }

    /// A view of the collected environment variables.
    pub fn vars(&self) -> &HashMap<String, String> {
        &self.vars
    }

    /// Consume the injector, returning the collected environment variables.
    pub fn into_vars(self) -> HashMap<String, String> {
        self.vars
    }
}

impl Injector for EnvironmentInjector {
    /// Set `key` to `value`, normalizing `key` before insertion.
    fn set(&mut self, key: &str, value: String) {
        self.vars.insert(normalize_key(key), value);
    }

    fn reserve(&mut self, additional: usize) {
        self.vars.reserve(additional);
    }
}

impl From<EnvironmentInjector> for HashMap<String, String> {
    fn from(injector: EnvironmentInjector) -> Self {
        injector.vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_key_rules() {
        assert_eq!(normalize_key(""), "_");
        assert_eq!(normalize_key("x-b3-traceid"), "X_B3_TRACEID");
        assert_eq!(normalize_key("traceparent"), "TRACEPARENT");
        assert_eq!(normalize_key("ALREADY_OK"), "ALREADY_OK");
        // Leading digit gets an underscore prefix.
        assert_eq!(normalize_key("1abc"), "_1ABC");
        assert_eq!(normalize_key("9"), "_9");
        // Non-alphanumerics (including non-ASCII) each become a single `_`.
        assert_eq!(normalize_key("a.b-c d"), "A_B_C_D");
        assert_eq!(normalize_key("ké"), "K_");
        assert_eq!(normalize_key("_leading"), "_LEADING");
    }

    #[test]
    fn is_normalized_key_rules() {
        assert!(is_normalized_key("TRACEPARENT"));
        assert!(is_normalized_key("_X_AMZN_TRACE_ID"));
        assert!(is_normalized_key("A1_B2"));

        assert!(!is_normalized_key(""));
        assert!(!is_normalized_key("1ABC")); // leading digit
        assert!(!is_normalized_key("lowercase"));
        assert!(!is_normalized_key("HAS-DASH"));
        assert!(!is_normalized_key("HAS SPACE"));
    }

    #[test]
    fn is_normalized_key_matches_normalize_fixpoint() {
        for input in ["traceparent", "x-b3-traceid", "1abc", "", "ké", "OK_1"] {
            let normalized = normalize_key(input);
            assert!(
                is_normalized_key(&normalized),
                "normalize_key({input:?}) = {normalized:?} should be normalized"
            );
        }
    }

    #[test]
    fn extractor_get_normalizes_lookup_key() {
        let extractor = EnvironmentExtractor::from_iter([
            ("TRACEPARENT".to_string(), "00-abc-def-01".to_string()),
            ("X_B3_TRACEID".to_string(), "trace".to_string()),
        ]);

        // Lookup keys are normalized, so any spelling that normalizes to the
        // stored key resolves.
        assert_eq!(extractor.get("traceparent"), Some("00-abc-def-01"));
        assert_eq!(extractor.get("TRACEPARENT"), Some("00-abc-def-01"));
        assert_eq!(extractor.get("x-b3-traceid"), Some("trace"));
        assert_eq!(extractor.get("missing"), None);
    }

    #[test]
    fn extractor_drops_non_normalized_keys() {
        let extractor = EnvironmentExtractor::from_iter([
            ("TRACEPARENT".to_string(), "value".to_string()),
            // Not normalized -- dropped at construction, matching Go/Python.
            ("traceparent".to_string(), "lowercase".to_string()),
            ("has-dash".to_string(), "value".to_string()),
        ]);

        let mut keys = extractor.keys();
        keys.sort_unstable();
        assert_eq!(keys, vec!["TRACEPARENT"]);
        assert_eq!(extractor.get("traceparent"), Some("value"));
    }

    #[test]
    fn injector_normalizes_and_collects() {
        let mut injector = EnvironmentInjector::new();
        injector.set("traceparent", "00-abc-def-01".to_string());
        injector.set("x-b3-traceid", "trace".to_string());

        let vars = injector.into_vars();
        assert_eq!(vars.get("TRACEPARENT"), Some(&"00-abc-def-01".to_string()));
        assert_eq!(vars.get("X_B3_TRACEID"), Some(&"trace".to_string()));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn injector_set_overwrites_normalized_collision() {
        let mut injector = EnvironmentInjector::new();
        injector.set("trace-parent", "first".to_string());
        // Normalizes to the same key, so it overwrites.
        injector.set("trace_parent", "second".to_string());

        let vars = injector.into_vars();
        assert_eq!(vars.get("TRACE_PARENT"), Some(&"second".to_string()));
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn injector_into_hashmap() {
        let mut injector = EnvironmentInjector::new();
        injector.set("traceparent", "value".to_string());
        let vars: HashMap<String, String> = injector.into();
        assert_eq!(vars.get("TRACEPARENT"), Some(&"value".to_string()));
    }
}
