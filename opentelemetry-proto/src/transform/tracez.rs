#[cfg(feature = "gen-protoc")]
mod grpcio {
    use opentelemetry_api::trace::{self, Event};
    use opentelemetry_sdk::export::trace::SpanData;

    use crate::transform::common::{grpcio::Attributes, to_nanos};
    use crate::{
        grpcio::trace::Status_StatusCode,
        proto::grpcio::{
            trace::{Span_Event, Status},
            tracez::{ErrorData, LatencyData, RunningData},
        },
    };

    impl From<SpanData> for LatencyData {
        fn from(span_data: SpanData) -> Self {
            LatencyData {
                traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
                spanid: span_data.span_context.span_id().to_bytes().to_vec(),
                parentid: span_data.parent_span_id.to_bytes().to_vec(),
                starttime: to_nanos(span_data.start_time),
                endtime: to_nanos(span_data.end_time),
                attributes: Attributes::from(span_data.attributes).0,
                events: span_data.events.iter().cloned().map(Into::into).collect(),
                links: span_data.links.iter().cloned().map(Into::into).collect(),
                ..Default::default()
            }
        }
    }

    impl From<SpanData> for ErrorData {
        fn from(span_data: SpanData) -> Self {
            ErrorData {
                traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
                spanid: span_data.span_context.span_id().to_bytes().to_vec(),
                parentid: span_data.parent_span_id.to_bytes().to_vec(),
                starttime: to_nanos(span_data.start_time),
                attributes: Attributes::from(span_data.attributes).0,
                events: span_data.events.iter().cloned().map(Into::into).collect(),
                links: span_data.links.iter().cloned().map(Into::into).collect(),
                status: ::protobuf::SingularPtrField::from(match span_data.status {
                    trace::Status::Error {
                        description: message,
                    } => Some(Status {
                        message: message.to_string(),
                        code: Status_StatusCode::STATUS_CODE_ERROR,
                        ..Default::default()
                    }),
                    _ => None,
                }),
                ..Default::default()
            }
        }
    }

    impl From<SpanData> for RunningData {
        fn from(span_data: SpanData) -> Self {
            RunningData {
                traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
                spanid: span_data.span_context.span_id().to_bytes().to_vec(),
                parentid: span_data.parent_span_id.to_bytes().to_vec(),
                starttime: to_nanos(span_data.start_time),
                attributes: Attributes::from(span_data.attributes).0,
                events: span_data.events.iter().cloned().map(Into::into).collect(),
                links: span_data.links.iter().cloned().map(Into::into).collect(),
                ..Default::default()
            }
        }
    }

    impl From<Event> for Span_Event {
        fn from(event: Event) -> Self {
            Span_Event {
                time_unix_nano: to_nanos(event.timestamp),
                name: event.name.to_string(),
                attributes: Attributes::from(event.attributes).0,
                dropped_attributes_count: event.dropped_attributes_count,
                ..Default::default()
            }
        }
    }
}
