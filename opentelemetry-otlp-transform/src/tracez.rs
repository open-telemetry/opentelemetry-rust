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

#[cfg(test)]
mod tests {
    use super::{
        event_to_span_event, span_to_error_data, span_to_latency_data, span_to_running_data,
    };
    use opentelemetry::{
        trace::{
            Event, Link, SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
        },
        InstrumentationScope, KeyValue,
    };
    use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks};
    use std::time::{Duration, UNIX_EPOCH};

    fn span_data(status: Status) -> SpanData {
        let mut events = SpanEvents::default();
        events.events.push(Event::new(
            "tracez event",
            UNIX_EPOCH + Duration::from_nanos(150),
            vec![KeyValue::new("event.key", "event-value")],
            1,
        ));

        let mut links = SpanLinks::default();
        links.links.push(Link::new(
            SpanContext::new(
                TraceId::from(0x2233_4455_6677_8899_aabb_ccdd_eeff_0011),
                SpanId::from(0x2030_4050_6070_8090),
                TraceFlags::SAMPLED,
                true,
                TraceState::default(),
            ),
            vec![KeyValue::new("link.key", "link-value")],
            2,
        ));

        SpanData {
            span_context: SpanContext::new(
                TraceId::from(0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00),
                SpanId::from(0x1020_3040_5060_7080),
                TraceFlags::SAMPLED,
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from(0x8877_6655_4433_2211),
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "tracez span".into(),
            start_time: UNIX_EPOCH + Duration::from_nanos(100),
            end_time: UNIX_EPOCH + Duration::from_nanos(200),
            attributes: vec![KeyValue::new("span.key", "span-value")],
            dropped_attributes_count: 0,
            events,
            links,
            status,
            instrumentation_scope: InstrumentationScope::default(),
        }
    }

    #[test]
    fn public_tracez_conversions_preserve_span_data() {
        let latency = span_to_latency_data(span_data(Status::Unset));
        assert_eq!(latency.starttime, 100);
        assert_eq!(latency.endtime, 200);
        assert_eq!(latency.attributes[0].key, "span.key");
        assert_eq!(latency.events[0].name, "tracez event");
        assert_eq!(latency.links.len(), 1);

        let error = span_to_error_data(span_data(Status::Error {
            description: "failed".into(),
        }));
        let status = error.status.expect("error status");
        assert_eq!(status.code, 2);
        assert_eq!(status.message, "failed");

        let non_error = span_to_error_data(span_data(Status::Unset));
        assert!(non_error.status.is_none());

        let running = span_to_running_data(span_data(Status::Unset));
        assert_eq!(running.starttime, 100);
        assert_eq!(running.events[0].name, "tracez event");
        assert_eq!(running.links.len(), 1);

        let event = event_to_span_event(Event::new(
            "standalone event",
            UNIX_EPOCH + Duration::from_nanos(300),
            vec![KeyValue::new("event.key", "event-value")],
            3,
        ));
        assert_eq!(event.time_unix_nano, 300);
        assert_eq!(event.name, "standalone event");
        assert_eq!(event.attributes[0].key, "event.key");
        assert_eq!(event.dropped_attributes_count, 3);
    }
}
