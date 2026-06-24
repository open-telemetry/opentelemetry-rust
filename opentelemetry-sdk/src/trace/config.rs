//! SDK Configuration
//!
//! Configuration represents the global tracing configuration, overrides
//! can be set for the default OpenTelemetry limits and Sampler.
use crate::trace::{span_limit::SpanLimits, IdGenerator, RandomIdGenerator, Sampler, ShouldSample};
use crate::Resource;
use opentelemetry::otel_warn;
use std::borrow::Cow;
use std::env;
use std::str::FromStr;

/// Tracer configuration
#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    /// The sampler that the sdk should use
    pub sampler: Box<dyn ShouldSample>,

    /// The id generator that the sdk should use
    pub id_generator: Box<dyn IdGenerator>,

    /// span limits
    pub span_limits: SpanLimits,

    /// Contains attributes representing an entity that produces telemetry.
    pub resource: Cow<'static, Resource>,
}

impl Default for Config {
    /// Create default global sdk configuration.
    fn default() -> Self {
        let mut config = Config {
            sampler: Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn))),
            id_generator: Box::<RandomIdGenerator>::default(),
            span_limits: SpanLimits::default(),
            resource: Cow::Owned(Resource::builder().build()),
        };

        // `OTEL_ATTRIBUTE_COUNT_LIMIT` is the general fallback for the maximum
        // span attribute count; the span-specific
        // `OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT` takes precedence when both are set.
        // It does not affect the per-event / per-link attribute limits, which
        // have their own defaults per the spec.
        // See https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/#attribute-limits
        if let Some(max_attributes) = env::var("OTEL_ATTRIBUTE_COUNT_LIMIT")
            .ok()
            .and_then(|count_limit| u32::from_str(&count_limit).ok())
        {
            config.span_limits.max_attributes_per_span = max_attributes;
        }

        if let Some(max_attributes_per_span) = env::var("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT")
            .ok()
            .and_then(|count_limit| u32::from_str(&count_limit).ok())
        {
            config.span_limits.max_attributes_per_span = max_attributes_per_span;
        }

        if let Some(max_events_per_span) = env::var("OTEL_SPAN_EVENT_COUNT_LIMIT")
            .ok()
            .and_then(|max_events| u32::from_str(&max_events).ok())
        {
            config.span_limits.max_events_per_span = max_events_per_span;
        }

        if let Some(max_links_per_span) = env::var("OTEL_SPAN_LINK_COUNT_LIMIT")
            .ok()
            .and_then(|max_links| u32::from_str(&max_links).ok())
        {
            config.span_limits.max_links_per_span = max_links_per_span;
        }

        let sampler_arg = env::var("OTEL_TRACES_SAMPLER_ARG").ok();
        if let Ok(sampler) = env::var("OTEL_TRACES_SAMPLER") {
            config.sampler = match sampler.as_str() {
                "always_on" => Box::new(Sampler::AlwaysOn),
                "always_off" => Box::new(Sampler::AlwaysOff),
                "traceidratio" => {
                    let ratio = sampler_arg.as_ref().and_then(|r| r.parse::<f64>().ok());
                    if let Some(r) = ratio {
                        Box::new(Sampler::TraceIdRatioBased(r))
                    } else {
                        otel_warn!(
                            name: "TracerProvider.Config.InvalidSamplerArgument",
                            message = "OTEL_TRACES_SAMPLER is set to 'traceidratio' but OTEL_TRACES_SAMPLER_ARG environment variable is missing or invalid. OTEL_TRACES_SAMPLER_ARG must be a valid float between 0.0 and 1.0 representing the desired sampling probability (0.0 = no traces sampled, 1.0 = all traces sampled, 0.5 = 50% of traces sampled). Falling back to default ratio: 1.0 (100% sampling)",
                            otel_traces_sampler_arg = format!("{:?}", sampler_arg)
                        );
                        Box::new(Sampler::TraceIdRatioBased(1.0))
                    }
                }
                "parentbased_always_on" => {
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
                }
                "parentbased_always_off" => {
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOff)))
                }
                "parentbased_traceidratio" => {
                    let ratio = sampler_arg.as_ref().and_then(|r| r.parse::<f64>().ok());
                    if let Some(r) = ratio {
                        Box::new(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                            r,
                        ))))
                    } else {
                        otel_warn!(
                            name: "TracerProvider.Config.InvalidSamplerArgument",
                            message = "OTEL_TRACES_SAMPLER is set to 'parentbased_traceidratio' but OTEL_TRACES_SAMPLER_ARG environment variable is missing or invalid. OTEL_TRACES_SAMPLER_ARG must be a valid float between 0.0 and 1.0 representing the desired sampling probability (0.0 = no traces sampled, 1.0 = all traces sampled, 0.5 = 50% of traces sampled). Falling back to default ratio: 1.0 (100% sampling)",
                            otel_traces_sampler_arg = format!("{:?}", sampler_arg)
                        );
                        Box::new(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                            1.0,
                        ))))
                    }
                }
                "parentbased_jaeger_remote" => {
                    otel_warn!(
                        name: "TracerProvider.Config.UnsupportedSampler",
                        message = "OTEL_TRACES_SAMPLER is set to 'parentbased_jaeger_remote' which is not implemented in this SDK version. Using fallback sampler: ParentBased(AlwaysOn). Configure an alternative sampler using OTEL_TRACES_SAMPLER"
                    );
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
                }
                "jaeger_remote" => {
                    otel_warn!(
                        name: "TracerProvider.Config.UnsupportedSampler",
                        message = "OTEL_TRACES_SAMPLER is set to 'jaeger_remote' which is implemented in this SDK version. Using fallback sampler: ParentBased(AlwaysOn). Configure an alternative sampler using OTEL_TRACES_SAMPLER"
                    );
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
                }
                "xray" => {
                    otel_warn!(
                        name: "TracerProvider.Config.UnsupportedSampler",
                        message = "OTEL_TRACES_SAMPLER is set to 'xray'. AWS X-Ray sampler is not implemented in this SDK version. Using fallback sampler: ParentBased(AlwaysOn). Configure an alternative sampler using OTEL_TRACES_SAMPLER"
                    );
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
                }
                s => {
                    otel_warn!(
                        name: "TracerProvider.Config.InvalidSamplerType",
                        message = format!(
                            "Unrecognized sampler type '{}' in OTEL_TRACES_SAMPLER environment variable. Valid values are: always_on, always_off, traceidratio, parentbased_always_on, parentbased_always_off, parentbased_traceidratio. Using fallback sampler: ParentBased(AlwaysOn)",
                            s
                        ),
                    );
                    Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
                }
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::trace::span_limit::{
        DEFAULT_MAX_ATTRIBUTES_PER_EVENT, DEFAULT_MAX_ATTRIBUTES_PER_LINK,
        DEFAULT_MAX_ATTRIBUTES_PER_SPAN,
    };

    #[test]
    fn otel_attribute_count_limit_sets_span_attribute_limit() {
        temp_env::with_vars(
            [
                ("OTEL_ATTRIBUTE_COUNT_LIMIT", Some("64")),
                ("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT", None::<&str>),
            ],
            || {
                let config = Config::default();
                assert_eq!(config.span_limits.max_attributes_per_span, 64);
                // The general limit must not touch the per-event / per-link
                // attribute limits, which keep their own defaults.
                assert_eq!(
                    config.span_limits.max_attributes_per_event,
                    DEFAULT_MAX_ATTRIBUTES_PER_EVENT
                );
                assert_eq!(
                    config.span_limits.max_attributes_per_link,
                    DEFAULT_MAX_ATTRIBUTES_PER_LINK
                );
            },
        );
    }

    #[test]
    fn span_attribute_count_limit_overrides_general_limit() {
        temp_env::with_vars(
            [
                ("OTEL_ATTRIBUTE_COUNT_LIMIT", Some("64")),
                ("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT", Some("32")),
            ],
            || {
                let config = Config::default();
                assert_eq!(config.span_limits.max_attributes_per_span, 32);
            },
        );
    }

    #[test]
    fn default_span_attribute_limit_when_unset() {
        temp_env::with_vars(
            [
                ("OTEL_ATTRIBUTE_COUNT_LIMIT", None::<&str>),
                ("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT", None::<&str>),
            ],
            || {
                let config = Config::default();
                assert_eq!(
                    config.span_limits.max_attributes_per_span,
                    DEFAULT_MAX_ATTRIBUTES_PER_SPAN
                );
            },
        );
    }
}
