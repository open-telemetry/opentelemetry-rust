use std::collections::HashSet;

use opentelemetry::{
    otel_warn,
    propagation::{TextMapCompositePropagator, TextMapPropagator},
};

use crate::propagation::{BaggagePropagator, TraceContextPropagator};

const OTEL_PROPAGATORS: &str = "OTEL_PROPAGATORS";
const OTEL_PROPAGATORS_DEFAULT: &str = "tracecontext,baggage";

/// Parses a text map propagator configuration from an environment variable value string.
///
/// If `value` is `None`, empty string, or whitespace-only, the specification default
/// (`"tracecontext,baggage"`) is used.
pub(crate) fn propagator_from_env_value(value: Option<&str>) -> TextMapCompositePropagator {
    let raw = match value {
        Some(s) if !s.trim().is_empty() => s,
        _ => OTEL_PROPAGATORS_DEFAULT,
    };

    let mut propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>> = Vec::new();
    let mut seen = HashSet::new();

    for entry in raw.split(',') {
        let name = entry.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }

        match name.as_str() {
            "tracecontext" => {
                if seen.insert("tracecontext") {
                    propagators.push(Box::new(TraceContextPropagator::new()));
                }
            }
            "baggage" => {
                if seen.insert("baggage") {
                    propagators.push(Box::new(BaggagePropagator::new()));
                }
            }
            "none" => {
                // "none" configures no propagator. It adds nothing to the composite.
            }
            _ => {
                otel_warn!(
                    name: "TextMapPropagator.Config.UnsupportedPropagator",
                    message = format!(
                        "Unrecognized or unsupported propagator '{name}' in OTEL_PROPAGATORS environment variable. Supported values are: tracecontext, baggage, none. Ignoring.",
                    ),
                    propagator = name.as_str(),
                );
            }
        }
    }

    TextMapCompositePropagator::new(propagators)
}

/// Reads the `OTEL_PROPAGATORS` environment variable and constructs the configured
/// [`TextMapCompositePropagator`].
pub(crate) fn build_propagator_from_env() -> TextMapCompositePropagator {
    match std::env::var(OTEL_PROPAGATORS) {
        Ok(val) => propagator_from_env_value(Some(&val)),
        Err(std::env::VarError::NotPresent) => propagator_from_env_value(None),
        Err(std::env::VarError::NotUnicode(err)) => {
            otel_warn!(
                name: "TextMapPropagator.Config.InvalidEnvVarEncoding",
                message = format!(
                    "OTEL_PROPAGATORS environment variable is not valid Unicode: {:?}. Falling back to default: '{}'",
                    err,
                    OTEL_PROPAGATORS_DEFAULT
                ),
            );
            propagator_from_env_value(None)
        }
    }
}

/// Configures the process-global text map propagator from the `OTEL_PROPAGATORS` environment variable.
///
/// If `OTEL_PROPAGATORS` is unset or empty, the default `"tracecontext,baggage"` is used, per the
/// OpenTelemetry specification.
///
/// # Supported Values
///
/// The following values are supported:
/// - `"tracecontext"`: [`TraceContextPropagator`]
/// - `"baggage"`: [`BaggagePropagator`]
/// - `"none"`: produces no propagation (adds no propagators to the composite)
///
/// Multiple propagators can be specified as a comma-separated list and will be combined into a
/// [`TextMapCompositePropagator`]. Values are case-insensitive, surrounding whitespace is trimmed,
/// and duplicate propagator names are deduplicated.
///
/// Unrecognized propagator names are ignored with an internal warning.
///
/// # Global Side Effect
///
/// Calling this function sets the process-global text map propagator via
/// [`opentelemetry::global::set_text_map_propagator`]. This is an explicit helper and is not
/// called automatically by provider builders.
pub fn set_global_text_map_propagator_from_env() {
    let propagator = build_propagator_from_env();
    opentelemetry::global::set_text_map_propagator(propagator);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn get_fields(propagator: &TextMapCompositePropagator) -> HashSet<&str> {
        propagator.fields().collect()
    }

    #[test]
    fn test_unset_env_value_uses_default() {
        let propagator = propagator_from_env_value(None);
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_empty_env_value_uses_default() {
        let propagator = propagator_from_env_value(Some(""));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_whitespace_only_env_value_uses_default() {
        let propagator = propagator_from_env_value(Some("   \t  "));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_tracecontext_only() {
        let propagator = propagator_from_env_value(Some("tracecontext"));
        let fields = get_fields(&propagator);
        assert_eq!(fields, HashSet::from(["traceparent", "tracestate"]));
    }

    #[test]
    fn test_baggage_only() {
        let propagator = propagator_from_env_value(Some("baggage"));
        let fields = get_fields(&propagator);
        assert_eq!(fields, HashSet::from(["baggage"]));
    }

    #[test]
    fn test_tracecontext_and_baggage() {
        let propagator = propagator_from_env_value(Some("tracecontext,baggage"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_reversed_ordering() {
        let propagator = propagator_from_env_value(Some("baggage,tracecontext"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_surrounding_whitespace_trimmed() {
        let propagator = propagator_from_env_value(Some("  tracecontext  ,   baggage   "));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_case_insensitivity() {
        let propagator = propagator_from_env_value(Some("TRACECONTEXT, Baggage"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_deduplication() {
        let propagator =
            propagator_from_env_value(Some("tracecontext, baggage, tracecontext, baggage"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_none_produces_no_fields() {
        let propagator = propagator_from_env_value(Some("none"));
        let fields = get_fields(&propagator);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_none_case_insensitive() {
        let propagator = propagator_from_env_value(Some("NONE"));
        let fields = get_fields(&propagator);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_none_with_tracecontext() {
        let propagator = propagator_from_env_value(Some("tracecontext, none"));
        let fields = get_fields(&propagator);
        assert_eq!(fields, HashSet::from(["traceparent", "tracestate"]));
    }

    #[test]
    fn test_none_with_baggage() {
        let propagator = propagator_from_env_value(Some("none, baggage"));
        let fields = get_fields(&propagator);
        assert_eq!(fields, HashSet::from(["baggage"]));
    }

    #[test]
    fn test_none_repeated() {
        let propagator = propagator_from_env_value(Some("none, none"));
        let fields = get_fields(&propagator);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_unknown_propagator_ignored() {
        let propagator = propagator_from_env_value(Some("unknown_prop, b3, jaeger"));
        let fields = get_fields(&propagator);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_mixture_supported_and_unsupported() {
        let propagator =
            propagator_from_env_value(Some("unknown1, tracecontext, b3, baggage, xray"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_consecutive_and_trailing_commas() {
        let propagator = propagator_from_env_value(Some(",,tracecontext,,baggage,"));
        let fields = get_fields(&propagator);
        assert_eq!(
            fields,
            HashSet::from(["traceparent", "tracestate", "baggage"])
        );
    }

    #[test]
    fn test_build_propagator_from_env_with_temp_env() {
        temp_env::with_var(OTEL_PROPAGATORS, Some("baggage"), || {
            let propagator = build_propagator_from_env();
            let fields = get_fields(&propagator);
            assert_eq!(fields, HashSet::from(["baggage"]));
        });

        temp_env::with_var_unset(OTEL_PROPAGATORS, || {
            let propagator = build_propagator_from_env();
            let fields = get_fields(&propagator);
            assert_eq!(
                fields,
                HashSet::from(["traceparent", "tracestate", "baggage"])
            );
        });
    }
}
