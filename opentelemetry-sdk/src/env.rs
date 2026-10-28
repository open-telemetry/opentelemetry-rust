//! Parsing of SDK-wide environment variables.

use opentelemetry::otel_warn;

const OTEL_SDK_DISABLED: &str = "OTEL_SDK_DISABLED";

/// Checks the value of a boolean environment variable.
///
/// Returns `true` only if the environment variable is set to a case-insensitive
/// `"true"`.
fn check_bool_env_var(var_name: &str) -> bool {
    let Ok(value) = std::env::var(var_name) else {
        return false;
    };

    if value.eq_ignore_ascii_case("true") {
        return true;
    }
    if value.is_empty() || value.eq_ignore_ascii_case("false") {
        return false;
    }
    otel_warn!(
        name: "EnvVarInvalid",
        message = format!("{} is not a valid boolean, so it is being ignored and the default value `false` is used.", var_name),
        value = value.as_str()
    );
    false
}

/// Returns `true` if the SDK is disabled through `OTEL_SDK_DISABLED`.
pub(crate) fn sdk_disabled() -> bool {
    check_bool_env_var(OTEL_SDK_DISABLED)
}

#[cfg(test)]
mod tests {
    use super::sdk_disabled;

    #[test]
    #[ignore = "modifies OTEL_SDK_DISABLED env var which can affect other tests"]
    fn otel_sdk_disabled_env_parsing() {
        for value in ["true", "TRUE", "True", "tRuE"] {
            temp_env::with_var("OTEL_SDK_DISABLED", Some(value), || {
                assert!(sdk_disabled(), "{value:?} should disable the SDK");
            });
        }

        // whitespace padded values are not a valid `"true"` either
        for value in [
            None,
            Some(""),
            Some(" "),
            Some("false"),
            Some("FALSE"),
            Some("yes"),
            Some(" true "),
            Some("true "),
        ] {
            temp_env::with_var("OTEL_SDK_DISABLED", value, || {
                assert!(!sdk_disabled(), "{value:?} should leave the SDK enabled");
            });
        }
    }
}
