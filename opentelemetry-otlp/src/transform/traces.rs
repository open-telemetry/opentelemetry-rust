use crate::proto::resource::Resource;
use crate::proto::trace::{
    InstrumentationLibrarySpans, ResourceSpans, Span, Span_Event, Span_Link, Span_SpanKind, Status,
    Status_StatusCode,
};
use crate::transform::common::{to_nanos, Attributes};
use opentelemetry::api::trace::{Link, SpanKind, StatusCode};
use opentelemetry::exporter::trace::SpanData;
use protobuf::reflect::ProtobufValue;
use protobuf::{RepeatedField, SingularPtrField};

impl From<SpanKind> for Span_SpanKind {
    fn from(span_kind: SpanKind) -> Self {
        match span_kind {
            SpanKind::Client => Span_SpanKind::SPAN_KIND_CLIENT,
            SpanKind::Consumer => Span_SpanKind::SPAN_KIND_CONSUMER,
            SpanKind::Internal => Span_SpanKind::SPAN_KIND_INTERNAL,
            SpanKind::Producer => Span_SpanKind::SPAN_KIND_PRODUCER,
            SpanKind::Server => Span_SpanKind::SPAN_KIND_SERVER,
        }
    }
}

impl From<StatusCode> for Status_StatusCode {
    fn from(status_code: StatusCode) -> Self {
        match status_code {
            StatusCode::OK => Status_StatusCode::STATUS_CODE_OK,
            StatusCode::Canceled => Status_StatusCode::STATUS_CODE_CANCELLED,
            StatusCode::Unknown => Status_StatusCode::STATUS_CODE_UNKNOWN_ERROR,
            StatusCode::InvalidArgument => Status_StatusCode::STATUS_CODE_INVALID_ARGUMENT,
            StatusCode::DeadlineExceeded => Status_StatusCode::STATUS_CODE_DEADLINE_EXCEEDED,
            StatusCode::NotFound => Status_StatusCode::STATUS_CODE_NOT_FOUND,
            StatusCode::AlreadyExists => Status_StatusCode::STATUS_CODE_ALREADY_EXISTS,
            StatusCode::PermissionDenied => Status_StatusCode::STATUS_CODE_PERMISSION_DENIED,
            StatusCode::ResourceExhausted => Status_StatusCode::STATUS_CODE_RESOURCE_EXHAUSTED,
            StatusCode::FailedPrecondition => Status_StatusCode::STATUS_CODE_FAILED_PRECONDITION,
            StatusCode::Aborted => Status_StatusCode::STATUS_CODE_ABORTED,
            StatusCode::OutOfRange => Status_StatusCode::STATUS_CODE_OUT_OF_RANGE,
            StatusCode::Unimplemented => Status_StatusCode::STATUS_CODE_UNIMPLEMENTED,
            StatusCode::Internal => Status_StatusCode::STATUS_CODE_INTERNAL_ERROR,
            StatusCode::Unavailable => Status_StatusCode::STATUS_CODE_UNAVAILABLE,
            StatusCode::DataLoss => Status_StatusCode::STATUS_CODE_DATA_LOSS,
            StatusCode::Unauthenticated => Status_StatusCode::STATUS_CODE_UNAUTHENTICATED,
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
            trace_state: link.span_context().trace_state().header(),
            attributes: Attributes::from(link.attributes().clone()).0,
            dropped_attributes_count: 0,
            ..Default::default()
        }
    }
}

impl From<SpanData> for ResourceSpans {
    fn from(source_span: SpanData) -> Self {
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
                        trace_state: source_span.span_context.trace_state().header(),
                        parent_span_id: {
                            if source_span.parent_span_id.to_u64().is_non_zero() {
                                source_span.parent_span_id.to_u64().to_be_bytes().to_vec()
                            } else {
                                vec![]
                            }
                        },
                        name: source_span.name,
                        kind: source_span.span_kind.into(),
                        start_time_unix_nano: to_nanos(source_span.start_time),
                        end_time_unix_nano: to_nanos(source_span.end_time),
                        dropped_attributes_count: source_span.attributes.dropped_count(),
                        attributes: Attributes::from(source_span.attributes).0,
                        dropped_events_count: source_span.message_events.dropped_count(),
                        events: RepeatedField::from_vec(
                            source_span
                                .message_events
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
                        dropped_links_count: source_span.links.dropped_count(),
                        links: RepeatedField::from_vec(
                            source_span.links.into_iter().map(Into::into).collect(),
                        ),
                        status: SingularPtrField::some(Status {
                            code: Status_StatusCode::from(source_span.status_code),
                            message: source_span.status_message,
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
