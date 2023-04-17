use crate::transform::common::to_nanos;
use opentelemetry_api::trace::{Link, SpanId, SpanKind};
use opentelemetry_sdk::export::trace::SpanData;

#[cfg(feature = "gen-tonic")]
pub mod tonic {
    use super::*;
    use crate::proto::tonic::resource::v1::Resource;
    use crate::proto::tonic::trace::v1::{span, status, ResourceSpans, ScopeSpans, Span, Status};
    use crate::transform::common::tonic::Attributes;
    use opentelemetry_api::trace;

    impl From<SpanKind> for span::SpanKind {
        fn from(span_kind: SpanKind) -> Self {
            match span_kind {
                SpanKind::Client => span::SpanKind::Client,
                SpanKind::Consumer => span::SpanKind::Consumer,
                SpanKind::Internal => span::SpanKind::Internal,
                SpanKind::Producer => span::SpanKind::Producer,
                SpanKind::Server => span::SpanKind::Server,
            }
        }
    }

    impl From<&trace::Status> for status::StatusCode {
        fn from(status: &trace::Status) -> Self {
            match status {
                trace::Status::Ok => status::StatusCode::Ok,
                trace::Status::Unset => status::StatusCode::Unset,
                trace::Status::Error { .. } => status::StatusCode::Error,
            }
        }
    }

    impl From<Link> for span::Link {
        fn from(link: Link) -> Self {
            span::Link {
                trace_id: link.span_context.trace_id().to_bytes().to_vec(),
                span_id: link.span_context.span_id().to_bytes().to_vec(),
                trace_state: link.span_context.trace_state().header(),
                attributes: Attributes::from(link.attributes).0,
                dropped_attributes_count: link.dropped_attributes_count,
            }
        }
    }

    impl From<SpanData> for ResourceSpans {
        fn from(source_span: SpanData) -> Self {
            let span_kind: span::SpanKind = source_span.span_kind.into();
            ResourceSpans {
                resource: Some(Resource {
                    attributes: resource_attributes(&source_span.resource).0,
                    dropped_attributes_count: 0,
                }),
                schema_url: source_span
                    .resource
                    .schema_url()
                    .map(|url| url.to_string())
                    .unwrap_or_default(),
                scope_spans: vec![ScopeSpans {
                    schema_url: source_span
                        .instrumentation_lib
                        .schema_url
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    scope: Some(source_span.instrumentation_lib.into()),
                    spans: vec![Span {
                        trace_id: source_span.span_context.trace_id().to_bytes().to_vec(),
                        span_id: source_span.span_context.span_id().to_bytes().to_vec(),
                        trace_state: source_span.span_context.trace_state().header(),
                        parent_span_id: {
                            if source_span.parent_span_id != SpanId::INVALID {
                                source_span.parent_span_id.to_bytes().to_vec()
                            } else {
                                vec![]
                            }
                        },
                        name: source_span.name.into_owned(),
                        kind: span_kind as i32,
                        start_time_unix_nano: to_nanos(source_span.start_time),
                        end_time_unix_nano: to_nanos(source_span.end_time),
                        dropped_attributes_count: source_span.attributes.dropped_count(),
                        attributes: Attributes::from(source_span.attributes).0,
                        dropped_events_count: source_span.events.dropped_count(),
                        events: source_span
                            .events
                            .into_iter()
                            .map(|event| span::Event {
                                time_unix_nano: to_nanos(event.timestamp),
                                name: event.name.into(),
                                attributes: Attributes::from(event.attributes).0,
                                dropped_attributes_count: event.dropped_attributes_count,
                            })
                            .collect(),
                        dropped_links_count: source_span.links.dropped_count(),
                        links: source_span.links.into_iter().map(Into::into).collect(),
                        status: Some(Status {
                            code: status::StatusCode::from(&source_span.status).into(),
                            message: match source_span.status {
                                trace::Status::Error { description } => description.to_string(),
                                _ => Default::default(),
                            },
                        }),
                    }],
                }],
            }
        }
    }

    fn resource_attributes(resource: &opentelemetry_sdk::Resource) -> Attributes {
        resource
            .iter()
            .map(|(k, v)| opentelemetry_api::KeyValue::new(k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into()
    }
}

#[cfg(feature = "gen-protoc")]
pub mod grpcio {
    use super::*;
    use crate::proto::grpcio::resource::Resource;
    use crate::proto::grpcio::trace::{
        ResourceSpans, ScopeSpans, Span, Span_Event, Span_Link, Span_SpanKind, Status,
        Status_StatusCode,
    };
    use crate::transform::common::grpcio::Attributes;
    use opentelemetry_api::trace;
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

    impl From<&trace::Status> for Status_StatusCode {
        fn from(status: &trace::Status) -> Self {
            match status {
                trace::Status::Ok => Status_StatusCode::STATUS_CODE_OK,
                trace::Status::Unset => Status_StatusCode::STATUS_CODE_UNSET,
                trace::Status::Error { .. } => Status_StatusCode::STATUS_CODE_ERROR,
            }
        }
    }

    impl From<Link> for Span_Link {
        fn from(link: Link) -> Self {
            Span_Link {
                trace_id: link.span_context.trace_id().to_bytes().to_vec(),
                span_id: link.span_context.span_id().to_bytes().to_vec(),
                trace_state: link.span_context.trace_state().header(),
                attributes: Attributes::from(link.attributes).0,
                dropped_attributes_count: link.dropped_attributes_count,
                ..Default::default()
            }
        }
    }

    impl From<SpanData> for ResourceSpans {
        fn from(source_span: SpanData) -> Self {
            ResourceSpans {
                resource: SingularPtrField::from(Some(Resource {
                    attributes: resource_attributes(&source_span.resource).0,
                    dropped_attributes_count: 0,
                    ..Default::default()
                })),
                scope_spans: RepeatedField::from_vec(vec![ScopeSpans {
                    schema_url: source_span
                        .instrumentation_lib
                        .schema_url
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    scope: protobuf::SingularPtrField::some(source_span.instrumentation_lib.into()),
                    spans: RepeatedField::from_vec(vec![Span {
                        trace_id: source_span.span_context.trace_id().to_bytes().to_vec(),
                        span_id: source_span.span_context.span_id().to_bytes().to_vec(),
                        trace_state: source_span.span_context.trace_state().header(),
                        parent_span_id: {
                            if source_span.parent_span_id != SpanId::INVALID {
                                source_span.parent_span_id.to_bytes().to_vec()
                            } else {
                                vec![]
                            }
                        },
                        name: source_span.name.into_owned(),
                        kind: source_span.span_kind.into(),
                        start_time_unix_nano: to_nanos(source_span.start_time),
                        end_time_unix_nano: to_nanos(source_span.end_time),
                        dropped_attributes_count: source_span.attributes.dropped_count(),
                        attributes: Attributes::from(source_span.attributes).0,
                        dropped_events_count: source_span.events.dropped_count(),
                        events: RepeatedField::from_vec(
                            source_span
                                .events
                                .into_iter()
                                .map(|event| Span_Event {
                                    time_unix_nano: to_nanos(event.timestamp),
                                    name: event.name.into(),
                                    attributes: Attributes::from(event.attributes).0,
                                    dropped_attributes_count: event.dropped_attributes_count,
                                    ..Default::default()
                                })
                                .collect(),
                        ),
                        dropped_links_count: source_span.links.dropped_count(),
                        links: RepeatedField::from_vec(
                            source_span.links.into_iter().map(Into::into).collect(),
                        ),
                        status: SingularPtrField::some(Status {
                            code: Status_StatusCode::from(&source_span.status),
                            message: match source_span.status {
                                trace::Status::Error { description } => description.to_string(),
                                _ => Default::default(),
                            },
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            }
        }
    }

    fn resource_attributes(resource: &opentelemetry_sdk::Resource) -> Attributes {
        resource
            .iter()
            .map(|(k, v)| opentelemetry_api::KeyValue::new(k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into()
    }
}
