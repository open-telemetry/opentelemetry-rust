use crate::proto::common::{AnyValue, ArrayValue, KeyValue};
use crate::proto::trace::{Span_Event, Span_Link, Span_SpanKind, Status, Status_StatusCode};
use crate::proto::tracez::{ErrorData, LatencyData, RunningData};
use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::sdk::trace::EvictedHashMap;
use opentelemetry::trace::{Event, Link, SpanKind, StatusCode};
use opentelemetry::util::take_or_else_clone;
use opentelemetry::{Array, Value};
use protobuf::RepeatedField;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl From<SpanData> for LatencyData {
    fn from(span_data: SpanData) -> Self {
        LatencyData {
            traceid: span_data.span_context.trace_id().to_byte_array().to_vec(),
            spanid: span_data.span_context.span_id().to_byte_array().to_vec(),
            parentid: span_data.parent_span_id.to_byte_array().to_vec(),
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
            traceid: span_data.span_context.trace_id().to_byte_array().to_vec(),
            spanid: span_data.span_context.span_id().to_byte_array().to_vec(),
            parentid: span_data.parent_span_id.to_byte_array().to_vec(),
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
            traceid: span_data.span_context.trace_id().to_byte_array().to_vec(),
            spanid: span_data.span_context.span_id().to_byte_array().to_vec(),
            parentid: span_data.parent_span_id.to_byte_array().to_vec(),
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
            StatusCode::Ok => Status_StatusCode::STATUS_CODE_OK,
            StatusCode::Unset => Status_StatusCode::STATUS_CODE_UNSET,
            StatusCode::Error => Status_StatusCode::STATUS_CODE_ERROR,
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
            dropped_attributes_count: link.dropped_attributes_count(),
            ..Default::default()
        }
    }
}

pub(crate) struct Attributes(pub(crate) ::protobuf::RepeatedField<crate::proto::common::KeyValue>);

impl From<EvictedHashMap> for Attributes {
    fn from(attributes: EvictedHashMap) -> Self {
        Attributes(RepeatedField::from_vec(
            attributes
                .into_iter()
                .map(|(key, value)| {
                    let mut kv: KeyValue = KeyValue::new();
                    kv.set_key(key.as_str().to_string());
                    kv.set_value(value.into());
                    kv
                })
                .collect(),
        ))
    }
}

impl From<Vec<opentelemetry::KeyValue>> for Attributes {
    fn from(kvs: Vec<opentelemetry::KeyValue>) -> Self {
        Attributes(RepeatedField::from_vec(
            kvs.into_iter()
                .map(|api_kv| {
                    let mut kv: KeyValue = KeyValue::new();
                    kv.set_key(api_kv.key.as_str().to_string());
                    kv.set_value(api_kv.value.into());
                    kv
                })
                .collect(),
        ))
    }
}

impl From<Value> for AnyValue {
    fn from(value: Value) -> Self {
        let mut any_value = AnyValue::new();
        match value {
            Value::Bool(val) => any_value.set_bool_value(val),
            Value::I64(val) => any_value.set_int_value(val),
            Value::F64(val) => any_value.set_double_value(val),
            Value::String(val) => any_value.set_string_value(val.into_owned()),
            Value::SharedString(val) => any_value.set_string_value(take_or_else_clone(val)),
            Value::Array(array) => any_value.set_array_value(match array {
                Array::Bool(vals) => array_into_proto(vals),
                Array::I64(vals) => array_into_proto(vals),
                Array::F64(vals) => array_into_proto(vals),
                Array::String(vals) => array_into_proto(vals),
            }),
        };

        any_value
    }
}

fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
where
    Value: From<T>,
{
    let values = RepeatedField::from_vec(
        vals.into_iter()
            .map(|val| AnyValue::from(Value::from(val)))
            .collect(),
    );

    let mut array_value = ArrayValue::new();
    array_value.set_values(values);
    array_value
}

pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}
