use crate::proto::resource::Resource;
use crate::proto::trace::{InstrumentationLibrarySpans, ResourceSpans, Span, Span_SpanKind};
use opentelemetry::api::SpanKind;
use opentelemetry::exporter::trace::SpanData;
use protobuf::{RepeatedField, SingularPtrField};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

impl From<SpanKind> for Span_SpanKind {
    fn from(span_kind: SpanKind) -> Self {
        match span_kind {
            SpanKind::Client => Span_SpanKind::CLIENT,
            SpanKind::Consumer => Span_SpanKind::CONSUMER,
            SpanKind::Internal => Span_SpanKind::INTERNAL,
            SpanKind::Producer => Span_SpanKind::PRODUCER,
            SpanKind::Server => Span_SpanKind::SERVER,
        }
    }
}

impl From<Arc<SpanData>> for ResourceSpans {
    fn from(source_span: Arc<SpanData>) -> Self {
        ResourceSpans {
            resource: SingularPtrField::from(Some(Resource {
                attributes: Default::default(),
                dropped_attributes_count: 0,
                unknown_fields: Default::default(),
                cached_size: Default::default(),
            })),
            instrumentation_library_spans: RepeatedField::from_vec(vec![
                InstrumentationLibrarySpans {
                    instrumentation_library: Default::default(),
                    spans: RepeatedField::from_vec(vec![Span {
                        trace_id: source_span
                            .span_context
                            .trace_id()
                            .to_u128()
                            .to_be_bytes()
                            .to_vec(),
                        span_id: source_span
                            .span_context
                            .span_id()
                            .to_u64()
                            .to_be_bytes()
                            .to_vec(),
                        trace_state: "".to_string(),
                        parent_span_id: source_span.parent_span_id.to_u64().to_be_bytes().to_vec(),
                        name: source_span.name.clone(),
                        kind: source_span.span_kind.clone().into(),
                        start_time_unix_nano: source_span
                            .start_time
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64,
                        end_time_unix_nano: source_span
                            .end_time
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64,
                        attributes: Default::default(),
                        dropped_attributes_count: 0,
                        events: Default::default(),
                        dropped_events_count: 0,
                        links: Default::default(),
                        dropped_links_count: 0,
                        status: Default::default(),
                        unknown_fields: Default::default(),
                        cached_size: Default::default(),
                    }]),
                    unknown_fields: Default::default(),
                    cached_size: Default::default(),
                },
            ]),
            unknown_fields: Default::default(),
            cached_size: Default::default(),
        }
    }
}
