//! SDK Configuration
//!
//! Configuration represents the global tracing configuration, overrides
//! can be set for the default OpenTelemetry limits and Sampler.
use crate::{sdk, sdk::trace::Sampler, trace::IdGenerator};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

/// Default trace configuration
pub fn config() -> Config {
    Config::default()
}

/// Tracer configuration
#[derive(Debug)]
pub struct Config {
    /// The sampler that the sdk should use
    pub default_sampler: Box<dyn sdk::trace::ShouldSample>,
    /// The id generator that the sdk should use
    pub id_generator: Box<dyn IdGenerator>,
    /// The max events that can be added to a `Span`.
    pub max_events_per_span: u32,
    /// The max attributes that can be added to a `Span`.
    pub max_attributes_per_span: u32,
    /// The max links that can be added to a `Span`.
    pub max_links_per_span: u32,
    /// Contains attributes representing an entity that produces telemetry.
    pub resource: Arc<sdk::Resource>,
}

impl Config {
    /// Specify the default sampler to be used.
    pub fn with_default_sampler<T: sdk::trace::ShouldSample + 'static>(
        mut self,
        sampler: T,
    ) -> Self {
        self.default_sampler = Box::new(sampler);
        self
    }

    /// Specify the id generator to be used.
    pub fn with_id_generator<T: IdGenerator + 'static>(mut self, id_generator: T) -> Self {
        self.id_generator = Box::new(id_generator);
        self
    }

    /// Specify the number of events to be recorded per span.
    pub fn with_max_events_per_span(mut self, max_events: u32) -> Self {
        self.max_events_per_span = max_events;
        self
    }

    /// Specify the number of attributes to be recorded per span.
    pub fn with_max_attributes_per_span(mut self, max_attributes: u32) -> Self {
        self.max_attributes_per_span = max_attributes;
        self
    }

    /// Specify the number of events to be recorded per span.
    pub fn with_max_links_per_span(mut self, max_links: u32) -> Self {
        self.max_links_per_span = max_links;
        self
    }

    /// Specify the attributes representing the entity that produces telemetry
    pub fn with_resource(mut self, resource: sdk::Resource) -> Self {
        self.resource = Arc::new(resource);
        self
    }
}

impl Default for Config {
    /// Create default global sdk configuration.
    fn default() -> Self {
        let mut config = Config {
            default_sampler: Box::new(Sampler::ParentBased(Box::new(Sampler::AlwaysOn))),
            id_generator: Box::new(sdk::trace::IdGenerator::default()),
            max_events_per_span: 128,
            max_attributes_per_span: 128,
            max_links_per_span: 128,
            resource: Arc::new(sdk::Resource::default()),
        };

        if let Some(max_attributes_per_span) = env::var("OTEL_SPAN_ATTRIBUTE_COUNT_LIMIT")
            .ok()
            .and_then(|count_limit| u32::from_str(&count_limit).ok())
        {
            config.max_attributes_per_span = max_attributes_per_span;
        }

        if let Some(max_events_per_span) = env::var("OTEL_SPAN_EVENT_COUNT_LIMIT")
            .ok()
            .and_then(|max_events| u32::from_str(&max_events).ok())
        {
            config.max_events_per_span = max_events_per_span;
        }

        if let Some(max_links_per_span) = env::var("OTEL_SPAN_LINK_COUNT_LIMIT")
            .ok()
            .and_then(|max_links| u32::from_str(&max_links).ok())
        {
            config.max_links_per_span = max_links_per_span;
        }

        config
    }
}
