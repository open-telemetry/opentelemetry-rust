use opentelemetry::propagation::{Extractor, Injector};
use std::collections::HashMap;
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
/// use opentelemetry::propagation::{Extractor, Injector};
/// use opentelemetry_sdk::propagation::EnvVarsCarrier;
///
/// // Builds the carrier, fetching the environment into the carrier mapping.
/// let mut carrier = EnvVarsCarrier::new();
///
/// // Looks for the normalized "FOO" value in the carrier mapping
/// let val = carrier.get("foo");
///
/// // Sets the value for normalized "FOO" to "bar", does NOT set env vars
/// carrier.set("foo", String::from("bar"));
/// ```
///
/// [POSIX.1-2024]: https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap08.html
#[derive(Debug, Clone)]
pub struct EnvVarsCarrier {
    map: HashMap<String, String>,
}

impl Default for EnvVarsCarrier {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvVarsCarrier {
    /// Create a new `EnvVarsCarrier` object, built from environment variables.
    /// Environment variables are fetched and normalized at construction time.
    pub fn new() -> Self {
        let map = std::env::vars().map(|(k, v)| (normalize(&k), v)).collect();

        Self { map }
    }

    /// Create a new `EnvVarsCarrier` object, internally empty. Useful for
    /// testing and for setting up environment mapping for subprocesses from
    /// scratch.
    pub fn new_empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Injector for EnvVarsCarrier {
    /// Set the value for the normalized key in the carrier mapping.
    /// Does NOT set environment variables, but may
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
    let mut bytes = name.chars().peekable();
    let needs_prefix = bytes.peek().is_some_and(|b| b.is_ascii_digit());

    needs_prefix
        .then_some('_')
        .into_iter()
        .chain(bytes.map(normalize_char))
        .collect()
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
    fn test_env_vars_carrier_injector() {
        let mut carrier = EnvVarsCarrier::new_empty();
        carrier.set("1foo.barᎁbaz", "bar".to_string());

        let entry = carrier.map.get("_1FOO_BAR_BAZ").unwrap();
        assert_eq!(entry, "bar");
    }

    #[test]
    fn test_env_vars_carrier_extractor() {
        let mut carrier = EnvVarsCarrier::new_empty();
        carrier
            .map
            .insert("FOO_BAR".to_string(), "value".to_string());

        assert_eq!(Extractor::get(&carrier, "foo.bar"), Some("value"));
    }

    #[test]
    fn test_env_vars_carrier_new() {
        let carrier = EnvVarsCarrier::new();
        let entry = carrier.get("ENV_VAR_CARRIER_TEST_VAR").unwrap();
        assert_eq!(entry, "test");
    }

    #[test]
    fn test_env_vars_carrier_default() {
        let carrier: EnvVarsCarrier = Default::default();
        let entry = carrier.get("ENV_VAR_CARRIER_TEST_VAR").unwrap();
        assert_eq!(entry, "test");
    }
}
