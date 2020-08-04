use crate::proto::common::{AnyValue, ArrayValue, KeyValue};
use crate::proto::resource::Resource;
use crate::proto::trace::{InstrumentationLibrarySpans, ResourceSpans, Span, Span_Event, Span_SpanKind, Status, Status_StatusCode};
use opentelemetry::api::{SpanKind, Value, StatusCode};
use opentelemetry::exporter::trace::SpanData;
use opentelemetry::sdk::EvictedHashMap;
use protobuf::reflect::ProtobufValue;
use protobuf::{RepeatedField, SingularPtrField};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

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

struct Attributes(::protobuf::RepeatedField<crate::proto::common::KeyValue>);

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

impl From<Vec<opentelemetry::api::KeyValue>> for Attributes {
    fn from(kvs: Vec<opentelemetry::api::KeyValue>) -> Self {
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
            Value::U64(val) => any_value.set_int_value(val as i64),
            Value::F64(val) => any_value.set_double_value(val),
            Value::String(val) => any_value.set_string_value(val),
            Value::Bytes(_val) => any_value.set_string_value("INVALID".to_string()),
            Value::Array(vals) => any_value.set_array_value({
                let mut array_value = ArrayValue::new();
                array_value.set_values(RepeatedField::from_vec(
                    vals.into_iter().map(|val| AnyValue::from(val)).collect(),
                ));
                array_value
            }),
        };

        any_value
    }
}

pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap_or_else(|_| Duration::from_secs(0)).as_nanos() as u64
}

impl From<Arc<SpanData>> for ResourceSpans {
    fn from(source_span: Arc<SpanData>) -> Self {
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
                        trace_state: "".to_string(),
                        parent_span_id: {
                            println!("Parent Span ID: {:?}", source_span.parent_span_id);
                            if !source_span.parent_span_id.to_u64().is_non_zero() {
                                vec![]
                            } else {
                                println!(
                                    "Parent Span to BigEndian Bytes: {:?}",
                                    source_span.parent_span_id.to_u64().to_be_bytes()
                                );
                                println!(
                                    "Parent Span bytes vec: {:?}",
                                    source_span.parent_span_id.to_u64().to_be_bytes().to_vec()
                                );
                                source_span.parent_span_id.to_u64().to_be_bytes().to_vec()
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
                        links: Default::default(),
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
