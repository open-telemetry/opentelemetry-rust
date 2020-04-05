//! # OpenTelemetry Sampler Interface
//!
//! ## Sampling
//!
//! Sampling is a mechanism to control the noise and overhead introduced by
//! OpenTelemetry by reducing the number of samples of traces collected and sent to
//! the backend.
//!
//! Sampling may be implemented on different stages of a trace collection.
//! OpenTelemetry API defines a `Sampler` interface that can be used at
//! instrumentation points by libraries to check the sampling `SamplingDecision` early and
//! optimize the amount of telemetry that needs to be collected.
//!
//! All other sampling algorithms may be implemented on SDK layer in exporters, or
//! even out of process in Agent or Collector.
//!
//! The OpenTelemetry API has two properties responsible for the data collection:
//!
//! * `is_recording` method on a `Span`. If `true` the current `Span` records
//!   tracing events (attributes, events, status, etc.), otherwise all tracing
//!   events are dropped. Users can use this property to determine if expensive
//!   trace events can be avoided. `SpanProcessor`s will receive
//!   all spans with this flag set. However, `SpanExporter`s will
//!   not receive them unless the `Sampled` flag was set.
//! * `Sampled` flag in `trace_flags` on `SpanContext`. This flag is propagated via
//!   the `SpanContext` to child Spans. For more details see the [W3C
//!   specification](https://w3c.github.io/trace-context/). This flag indicates that
//!   the `Span` has been `sampled` and will be exported. `SpanProcessor`s and
//!   `SpanExporter`s will receive spans with the `Sampled` flag set for
//!   processing.
//!
//! The flag combination `SampledFlag == false` and `is_recording` == true`
//! means that the current `Span` does record information, but most likely the child
//! `Span` will not.
//!
//! The flag combination `SampledFlag == true` and `IsRecording == false`
//! could cause gaps in the distributed trace, and because of this OpenTelemetry API
//! MUST NOT allow this combination.

use crate::api;

/// The `Sampler` interface allows implementations to provide samplers which will
/// return a sampling `SamplingResult` based on information that is typically
/// available just before the `Span` was created.
pub trait Sampler: Send + Sync + std::fmt::Debug {
    /// Returns the `SamplingDecision` for a `Span` to be created.
    #[allow(clippy::too_many_arguments)]
    fn should_sample(
        &self,
        parent_context: Option<&api::SpanContext>,
        trace_id: api::TraceId,
        span_id: api::SpanId,
        name: &str,
        span_kind: api::SpanKind,
        attributes: &[api::KeyValue],
        links: &[api::Link],
    ) -> SamplingResult;
}

/// The result of sampling logic for a given `Span`.
#[derive(Clone, Debug, PartialEq)]
pub struct SamplingResult {
    /// `SamplingDecision` reached by this result
    pub decision: SamplingDecision,
    /// Extra attributes added by this result
    pub attributes: Vec<api::KeyValue>,
}

/// Decision about whether or not to sample
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplingDecision {
    /// `is_recording() == false`, span will not be recorded and all events and
    /// attributes will be dropped.
    NotRecord,
    /// `is_recording() == true`, but `Sampled` flag MUST NOT be set.
    Record,
    /// `is_recording() == true` AND `Sampled` flag` MUST be set.
    RecordAndSampled,
}
