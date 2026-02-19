#[cfg(all(feature = "grpc-tonic", feature = "zpages"))]
mod tonic {
    use opentelemetry::trace::{Event, Status};
    use opentelemetry_sdk::trace::SpanData;

    use opentelemetry_proto::tonic::{
        trace::v1::{span::Event as SpanEvent, Status as SpanStatus},
        tracez::v1::{ErrorData, LatencyData, RunningData},
    };
    use crate::transform::common::{to_nanos, tonic::Attributes};
    use crate::transform::trace::tonic::link_to_proto_link;

    /// Converts span data to latency data for tracez.
    ///
    /// Note: This function is currently unused but is reserved for future zpages/tracez functionality.
    #[allow(dead_code)]
    pub(crate) fn span_data_to_latency_data(span_data: SpanData) -> LatencyData {
        LatencyData {
            traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
            spanid: span_data.span_context.span_id().to_bytes().to_vec(),
            parentid: span_data.parent_span_id.to_bytes().to_vec(),
            starttime: to_nanos(span_data.start_time),
            endtime: to_nanos(span_data.end_time),
            attributes: Attributes::from(span_data.attributes).0,
            events: span_data.events.iter().cloned().map(event_to_span_event).collect(),
            links: span_data.links.iter().cloned().map(link_to_proto_link).collect(),
        }
    }

    /// Converts span data to error data for tracez.
    ///
    /// Note: This function is currently unused but is reserved for future zpages/tracez functionality.
    #[allow(dead_code)]
    pub(crate) fn span_data_to_error_data(span_data: SpanData) -> ErrorData {
        ErrorData {
            traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
            spanid: span_data.span_context.span_id().to_bytes().to_vec(),
            parentid: span_data.parent_span_id.to_bytes().to_vec(),
            starttime: to_nanos(span_data.start_time),
            attributes: Attributes::from(span_data.attributes).0,
            events: span_data.events.iter().cloned().map(event_to_span_event).collect(),
            links: span_data.links.iter().cloned().map(link_to_proto_link).collect(),
            status: match span_data.status {
                Status::Error { description } => Some(SpanStatus {
                    message: description.to_string(),
                    code: 2,
                }),
                _ => None,
            },
        }
    }

    /// Converts span data to running data for tracez.
    ///
    /// Note: This function is currently unused but is reserved for future zpages/tracez functionality.
    #[allow(dead_code)]
    pub(crate) fn span_data_to_running_data(span_data: SpanData) -> RunningData {
        RunningData {
            traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
            spanid: span_data.span_context.span_id().to_bytes().to_vec(),
            parentid: span_data.parent_span_id.to_bytes().to_vec(),
            starttime: to_nanos(span_data.start_time),
            attributes: Attributes::from(span_data.attributes).0,
            events: span_data.events.iter().cloned().map(event_to_span_event).collect(),
            links: span_data.links.iter().cloned().map(link_to_proto_link).collect(),
        }
    }

    /// Converts an event to a span event for tracez.
    ///
    /// Note: This function is currently unused but is reserved for future zpages/tracez functionality.
    #[allow(dead_code)]
    pub(crate) fn event_to_span_event(event: Event) -> SpanEvent {
        SpanEvent {
            time_unix_nano: to_nanos(event.timestamp),
            name: event.name.to_string(),
            attributes: Attributes::from(event.attributes).0,
            dropped_attributes_count: event.dropped_attributes_count,
        }
    }
}
