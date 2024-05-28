use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use opentelemetry_sdk::export::trace::SpanData;
use std::io::Write;

pub fn pretty(writer: &mut dyn Write, spans: Vec<SpanData>) -> TraceResult<()> {
    let resource_spans: Vec<ResourceSpans> = spans.into_iter().map(ResourceSpans::from).collect();
    serde_json::to_writer_pretty(writer, &resource_spans)
        .map_err(|err| TraceError::Other(Box::new(err)))
}
