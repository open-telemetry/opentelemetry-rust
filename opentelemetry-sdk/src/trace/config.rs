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

/// Default trace configuration
#[deprecated(since = "0.23.0", note = "Use Config::default() instead")]
pub fn config() -> Config {
    Config::default()
}

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

impl Config {
    /// Specify the sampler to be used.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_sampler(...) instead."
    )]
    pub fn with_sampler<T: crate::trace::ShouldSample + 'static>(mut self, sampler: T) -> Self {
        self.sampler = Box::new(sampler);
        self
    }

    /// Specify the id generator to be used.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_id_generator(...) instead."
    )]
    pub fn with_id_generator<T: IdGenerator + 'static>(mut self, id_generator: T) -> Self {
        self.id_generator = Box::new(id_generator);
        self
    }

    /// Specify the maximum number of events that can be recorded per span.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_max_events_per_span(...) instead."
    )]
    pub fn with_max_events_per_span(mut self, max_events: u32) -> Self {
        self.span_limits.max_events_per_span = max_events;
        self
    }

    /// Specify the maximum number of attributes that can be recorded per span.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_max_attributes_per_span(...) instead."
    )]
    pub fn with_max_attributes_per_span(mut self, max_attributes: u32) -> Self {
        self.span_limits.max_attributes_per_span = max_attributes;
        self
    }

    /// Specify the maximum number of links that can be recorded per span.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_max_links_per_span(...) instead."
    )]
    pub fn with_max_links_per_span(mut self, max_links: u32) -> Self {
        self.span_limits.max_links_per_span = max_links;
        self
    }

    /// Specify the maximum number of attributes one event can have.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_max_attributes_per_event(...) instead."
    )]
    pub fn with_max_attributes_per_event(mut self, max_attributes: u32) -> Self {
        self.span_limits.max_attributes_per_event = max_attributes;
        self
    }

    /// Specify the maximum number of attributes one link can have.
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_max_attributes_per_link(...) instead."
    )]
    pub fn with_max_attributes_per_link(mut self, max_attributes: u32) -> Self {
        self.span_limits.max_attributes_per_link = max_attributes;
        self
    }

    /// Specify all limit via the span_limits
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_span_limits(...) instead."
    )]
    pub fn with_span_limits(mut self, span_limits: SpanLimits) -> Self {
        self.span_limits = span_limits;
        self
    }

    /// Specify the attributes representing the entity that produces telemetry
    #[deprecated(
        since = "0.27.1",
        note = "Config is becoming private. Please use Builder::with_resource(...) instead."
    )]
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resource = Cow::Owned(resource);
        self
    }
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
