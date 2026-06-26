use std::collections::HashMap;

use crate::propagation::{Extractor, Injector};

/// Propagates name-value pairs via environment variables.
///
/// This propagator provides a mechanism for propagating context information
/// across process boundaries using environment variables, usually for when
/// network protocols are not applicable.
///
/// Note that to comply with environment variable naming conventions, all keys
/// are normalized to be compatible with the [POSIX.1-2024] standard. The
/// normalization process follows these rules:
///
/// - uppercase ASCII letters
/// - replace non-alphanumeric/non-underscore characters with an underscore
/// - prefix name with an underscore if it otherwise starts with a digit
///
/// # Examples
/// ```
/// use opentelemetry::propagation::{Extractor, Injector, EnvVarsCarrier};
///
/// // Builds the carrier, fetching the environment into the carrier mapping.
/// // Filters any environment variables that are not already normalized.
/// let mut carrier = EnvVarsCarrier::from_env();
///
/// // Looks for the normalized "FOO" value in the carrier mapping
/// let val = carrier.get("foo");
///
/// // Sets the value for normalized "FOO" to "bar", does NOT set env vars
/// carrier.set("foo", String::from("bar"));
///
/// // Fetches the list of (normalized) keys
/// let keys = carrier.keys();  // vec!["FOO"]
/// ```
///
/// [POSIX.1-2024]: https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap08.html
#[derive(Debug, Clone)]
pub struct EnvVarsCarrier {
    map: HashMap<String, String>,
}

impl Default for EnvVarsCarrier {
    /// Create a new empty `EnvVarsCarrier` object.
    fn default() -> Self {
        Self::empty()
    }
}

impl EnvVarsCarrier {
    /// Create a new `EnvVarsCarrier` object, built from environment variables.
    /// Environment variables are fetched and normalized at construction time.
    pub fn from_env() -> Self {
        let map = std::env::vars().filter(|(k, _)| is_normalized(k)).collect();
        Self { map }
    }

    /// Create a new `EnvVarsCarrier` object, internally empty. Useful for
    /// testing and for setting up environment mapping for subprocesses from
    /// scratch.
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Injector for EnvVarsCarrier {
    /// Set the value for the normalized key in the carrier mapping.
    /// Does NOT set environment variables
    fn set(&mut self, key: &str, value: String) {
        self.map.insert(normalize(key), value);
    }
}

impl Extractor for EnvVarsCarrier {
    /// Get the value for the normalized key from the carrier mapping.
    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(&normalize(key)).map(|s| s.as_str())
    }

    /// List all of the internal mapping keys in their normalized form.
    fn keys(&self) -> Vec<&str> {
        self.map.keys().map(|k| k.as_str()).collect()
    }
}

#[inline(always)]
fn normalize_char(c: char) -> char {
    if c.is_ascii_alphanumeric() {
        c.to_ascii_uppercase()
    } else {
        '_'
    }
}

fn normalize(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let needs_prefix = chars.peek().is_some_and(|b| b.is_ascii_digit());

    needs_prefix
        .then_some('_')
        .into_iter()
        .chain(chars.map(normalize_char))
        .collect()
}

fn is_normalized(name: &str) -> bool {
    let mut chars = name.chars();

    if !chars
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
    {
        return false;
    }

    chars
        .find(|c| !c.is_ascii_uppercase() && !c.is_ascii_digit() && *c != '_')
        .is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("foo.bar"), "FOO_BAR");
        assert_eq!(normalize("3abc"), "_3ABC");
        assert_eq!(normalize("HELLO_WORLD"), "HELLO_WORLD");
        assert_eq!(normalize("a.b.c"), "A_B_C");
        assert_eq!(normalize("key with spaces"), "KEY_WITH_SPACES");
        assert_eq!(normalize("ⵕu⾫tⅭf⼤8"), "_U_T_F_8");
    }

    #[test]
    fn test_is_normalized_prefix() {
        assert!(is_normalized("ABC"));

        assert!(!is_normalized("3ABC")); // begins with number
        assert!(is_normalized("_3ABC"));

        assert!(!is_normalized("aBC")); // begins with lowercase letter
        assert!(is_normalized("ABC"));

        assert!(!is_normalized(".ABC")); // begins with non-alphanumeric/non-underscore character
        assert!(is_normalized("_ABC"));
    }

    #[test]
    fn test_is_normalized_body() {
        assert!(is_normalized("HELLO_WORLD"));

        assert!(!is_normalized("foo.bar"));
        assert!(!is_normalized("3abc"));
        assert!(!is_normalized("a.b.c"));
        assert!(!is_normalized("key with spaces"));
        assert!(!is_normalized("ⵕu⾫tⅭf⼤8"));
    }

    #[test]
    fn test_env_vars_carrier_injector() {
        let mut carrier = EnvVarsCarrier::empty();
        carrier.set("1foo.barᎁbaz", "bar".to_string());

        let entry = carrier.map.get("_1FOO_BAR_BAZ").unwrap();
        assert_eq!(entry, "bar");

        assert_eq!(carrier.keys(), vec!["_1FOO_BAR_BAZ"]);
    }

    #[test]
    fn test_env_vars_carrier_extractor() {
        let mut carrier = EnvVarsCarrier::empty();
        carrier
            .map
            .insert("FOO_BAR".to_string(), "value".to_string());

        assert_eq!(Extractor::get(&carrier, "foo.bar"), Some("value"));
    }

    #[test]
    fn test_env_vars_carrier_inject_and_extract() {
        let mut carrier = EnvVarsCarrier::empty();
        Injector::set(&mut carrier, "foo.bar", "value".to_string());
        assert_eq!(Extractor::get(&carrier, "foo.bar"), Some("value"));
        assert_eq!(carrier.keys(), vec!["FOO_BAR"]);
    }

    #[test]
    fn test_env_vars_carrier_from_env() {
        // refer to .cargo/config.toml for the environment variable definitions
        let carrier = EnvVarsCarrier::from_env();
        let entry = carrier.get("ENV_VAR_CARRIER_TEST_VAR").unwrap();
        assert_eq!(entry, "test");

        let vars = carrier.keys();
        assert!(!vars.contains(&"ENV-VAR-CARRIER-TEST-VAR2"));
    }

    #[test]
    fn test_env_vars_carrier_default() {
        let carrier: EnvVarsCarrier = Default::default();
        assert!(carrier.keys().is_empty());
    }
}
