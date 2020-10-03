use crate::proto::resource::Resource;
use crate::proto::trace::{
    InstrumentationLibrarySpans, ResourceSpans, Span, Span_Event, Span_Link, Span_SpanKind, Status,
    Status_StatusCode,
};
use crate::transform::common::{to_nanos, Attributes};
use opentelemetry::api::{Link, SpanKind, StatusCode};
use opentelemetry::exporter::trace::SpanData;
use protobuf::reflect::ProtobufValue;
use protobuf::{RepeatedField, SingularPtrField};
use std::sync::Arc;

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

impl From<StatusCode> for Status_StatusCode {
    fn from(status_code: StatusCode) -> Self {
        match status_code {
            StatusCode::OK => Status_StatusCode::Ok,
            StatusCode::Canceled => Status_StatusCode::Cancelled,
            StatusCode::Unknown => Status_StatusCode::UnknownError,
            StatusCode::InvalidArgument => Status_StatusCode::InvalidArgument,
            StatusCode::DeadlineExceeded => Status_StatusCode::DeadlineExceeded,
            StatusCode::NotFound => Status_StatusCode::NotFound,
            StatusCode::AlreadyExists => Status_StatusCode::AlreadyExists,
            StatusCode::PermissionDenied => Status_StatusCode::PermissionDenied,
            StatusCode::ResourceExhausted => Status_StatusCode::ResourceExhausted,
            StatusCode::FailedPrecondition => Status_StatusCode::FailedPrecondition,
            StatusCode::Aborted => Status_StatusCode::Aborted,
            StatusCode::OutOfRange => Status_StatusCode::OutOfRange,
            StatusCode::Unimplemented => Status_StatusCode::Unimplemented,
            StatusCode::Internal => Status_StatusCode::InternalError,
            StatusCode::Unavailable => Status_StatusCode::Unavailable,
            StatusCode::DataLoss => Status_StatusCode::DataLoss,
            StatusCode::Unauthenticated => Status_StatusCode::Unauthenticated,
        }
    }
}

impl From<Link> for Span_Link {
    fn from(link: Link) -> Self {
        Span_Link {
            trace_id: link
                .span_context()
                .trace_id()
                .to_u128()
                .to_be_bytes()
                .to_vec(),
            span_id: link
                .span_context()
                .span_id()
                .to_u64()
                .to_be_bytes()
                .to_vec(),
            // TODO Add TraceState to SpanContext API: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/api.md#spancontext
            trace_state: "".to_string(),
            attributes: Attributes::from(link.attributes().clone()).0,
            dropped_attributes_count: 0,
            ..Default::default()
        }
    }
}

impl From<&Arc<SpanData>> for ResourceSpans {
    fn from(source_span: &Arc<SpanData>) -> Self {
        ResourceSpans {
            resource: SingularPtrField::from(Some(Resource {
                attributes: Default::default(),
                dropped_attributes_count: 0,
                ..Default::default()
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
                        // TODO Add TraceState to SpanContext API: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/api.md#spancontext
                        trace_state: "".to_string(),
                        parent_span_id: {
                            if source_span.parent_span_id.to_u64().is_non_zero() {
                                source_span.parent_span_id.to_u64().to_be_bytes().to_vec()
                            } else {
                                vec![]
                            }
                        },
                        name: source_span.name.clone(),
                        kind: source_span.span_kind.clone().into(),
                        start_time_unix_nano: to_nanos(source_span.start_time),
                        end_time_unix_nano: to_nanos(source_span.end_time),
                        attributes: Attributes::from(source_span.attributes.clone()).0,
                        dropped_attributes_count: source_span.attributes.dropped_count(),
                        events: RepeatedField::from_vec(
                            source_span
                                .message_events
                                .clone()
                                .into_iter()
                                .map(|event| Span_Event {
                                    time_unix_nano: to_nanos(event.timestamp),
                                    name: event.name,
                                    attributes: Attributes::from(event.attributes).0,
                                    dropped_attributes_count: 0,
                                    ..Default::default()
                                })
                                .collect(),
                        ),
                        dropped_events_count: 0,
                        links: RepeatedField::from_vec(
                            source_span
                                .links
                                .clone()
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                        ),
                        dropped_links_count: 0,
                        status: SingularPtrField::some(Status {
                            code: Status_StatusCode::from(source_span.status_code.clone()),
                            message: source_span.status_message.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }
    }
}
