use opentelemetry::trace::{Event, Status};
use opentelemetry_proto::tonic::{
    trace::v1::{span::Event as SpanEvent, Status as SpanStatus},
    tracez::v1::{ErrorData, LatencyData, RunningData},
};
use opentelemetry_sdk::trace::SpanData;

use crate::{
    common::{to_nanos, tonic::keyvalues_to_proto},
    trace::tonic::link_to_proto,
};

/// Converts completed span data into zPages latency data.
pub fn span_to_latency_data(span_data: SpanData) -> LatencyData {
    LatencyData {
        traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
        spanid: span_data.span_context.span_id().to_bytes().to_vec(),
        parentid: span_data.parent_span_id.to_bytes().to_vec(),
        starttime: to_nanos(span_data.start_time),
        endtime: to_nanos(span_data.end_time),
        attributes: keyvalues_to_proto(span_data.attributes),
        events: span_data
            .events
            .into_iter()
            .map(event_to_span_event)
            .collect(),
        links: span_data.links.into_iter().map(link_to_proto).collect(),
    }
}

/// Converts completed span data into zPages error data.
pub fn span_to_error_data(span_data: SpanData) -> ErrorData {
    ErrorData {
        traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
        spanid: span_data.span_context.span_id().to_bytes().to_vec(),
        parentid: span_data.parent_span_id.to_bytes().to_vec(),
        starttime: to_nanos(span_data.start_time),
        attributes: keyvalues_to_proto(span_data.attributes),
        events: span_data
            .events
            .into_iter()
            .map(event_to_span_event)
            .collect(),
        links: span_data.links.into_iter().map(link_to_proto).collect(),
        status: match span_data.status {
            Status::Error { description } => Some(SpanStatus {
                message: description.to_string(),
                code: 2,
            }),
            _ => None,
        },
    }
}

/// Converts span data into zPages running data.
pub fn span_to_running_data(span_data: SpanData) -> RunningData {
    RunningData {
        traceid: span_data.span_context.trace_id().to_bytes().to_vec(),
        spanid: span_data.span_context.span_id().to_bytes().to_vec(),
        parentid: span_data.parent_span_id.to_bytes().to_vec(),
        starttime: to_nanos(span_data.start_time),
        attributes: keyvalues_to_proto(span_data.attributes),
        events: span_data
            .events
            .into_iter()
            .map(event_to_span_event)
            .collect(),
        links: span_data.links.into_iter().map(link_to_proto).collect(),
    }
}

/// Converts a span event into the protobuf representation used by zPages.
pub fn event_to_span_event(event: Event) -> SpanEvent {
    SpanEvent {
        time_unix_nano: to_nanos(event.timestamp),
        name: event.name.to_string(),
        attributes: keyvalues_to_proto(event.attributes),
        dropped_attributes_count: event.dropped_attributes_count,
    }
}
