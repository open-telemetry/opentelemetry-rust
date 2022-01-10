use std::time::{Duration, SystemTime, UNIX_EPOCH};

use protobuf::RepeatedField;

use opentelemetry::{
    Array,
    sdk::{
        export::trace::SpanData,
        trace::EvictedHashMap,
    },
    trace::{Event, Link, SpanKind, StatusCode}, Value,
};

use crate::proto::grpcio::{
    common::{AnyValue, ArrayValue, KeyValue},
    trace::{Span_Event, Span_Link, Span_SpanKind, Status, Status_StatusCode},
    tracez::{ErrorData, LatencyData, RunningData},
};
use crate::transform::common::{
    grpcio::Attributes,
    to_nanos,
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
            status: ::protobuf::SingularPtrField::from(Some(Status::from((
                span_data.status_code,
                span_data.status_message.to_string(),
            )))),
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

impl From<(StatusCode, String)> for Status {
    fn from((status_code, status_message): (StatusCode, String)) -> Self {
        Status {
            message: status_message,
            code: status_code.into(),
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