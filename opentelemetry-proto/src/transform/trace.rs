#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use crate::proto::tonic::resource::v1::Resource;
    use crate::proto::tonic::trace::v1::{span, status, ResourceSpans, ScopeSpans, Span, Status};
    use crate::transform::common::{
        to_nanos,
        tonic::{resource_attributes, Attributes},
    };
    use opentelemetry::trace;
    use opentelemetry::trace::{Link, SpanId, SpanKind};
    use opentelemetry_sdk::export::trace::SpanData;

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
                        dropped_attributes_count: source_span.dropped_attributes_count,
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
}

#[cfg(feature = "gen-grpcio")]
pub mod grpcio {
    use crate::proto::grpcio::resource::v1::Resource;
    use crate::proto::grpcio::trace::v1::{span, status, ResourceSpans, ScopeSpans, Span, Status};
    use crate::transform::common::{
        grpcio::{resource_attributes, Attributes},
        to_nanos,
    };
    use opentelemetry::trace;
    use opentelemetry::trace::{Link, SpanId, SpanKind};
    use opentelemetry_sdk::export::trace::SpanData;

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
}
