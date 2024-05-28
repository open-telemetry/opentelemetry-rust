use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use opentelemetry_sdk::export::trace::SpanData;
use std::io::Write;

/// serializes spans to stdout using pretty-printed JSON.
///
/// ```rust
/// use opentelemetry_stdout::SpanExporterBuilder;
///
/// let exporter = SpanExporterBuilder::default()
///     // pretty-print spans to stdout
///     .with_encoder(opentelemetry_stdout::pretty)
///     .build();
/// ```
pub fn pretty(writer: &mut dyn Write, spans: Vec<SpanData>) -> TraceResult<()> {
    let resource_spans: Vec<ResourceSpans> = spans.into_iter().map(ResourceSpans::from).collect();
    serde_json::to_writer_pretty(writer, &resource_spans)
        .map_err(|err| TraceError::Other(Box::new(err)))
}
