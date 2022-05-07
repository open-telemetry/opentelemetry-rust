use crate::transform::common::to_nanos;
use std::borrow::Cow;

#[cfg(feature = "gen-tonic")]
pub mod tonic {
    use std::collections::BTreeMap;

    use opentelemetry::sdk::logs::{Any, Severity};

    use crate::{
        tonic::{
            common::v1::{any_value::Value, AnyValue, ArrayValue, KeyValue, KeyValueList},
            logs::v1::{InstrumentationLibraryLogs, LogRecord, ResourceLogs, SeverityNumber},
            resource::v1::Resource,
        },
        transform::common::tonic::resource_attributes,
    };

    use super::*;

    impl From<Any> for Value {
        fn from(value: Any) -> Self {
            match value {
                Any::Double(f) => Value::DoubleValue(f),
                Any::Int(i) => Value::IntValue(i),
                Any::String(s) => Value::StringValue(s.into()),
                Any::Boolean(b) => Value::BoolValue(b),
                Any::ListAny(v) => Value::ArrayValue(ArrayValue {
                    values: v
                        .into_iter()
                        .map(|v| AnyValue {
                            value: Some(v.into()),
                        })
                        .collect(),
                }),
                Any::Map(m) => Value::KvlistValue(KeyValueList {
                    values: m
                        .into_iter()
                        .map(|(key, value)| KeyValue {
                            key: key.into_owned(),
                            value: Some(AnyValue {
                                value: Some(value.into()),
                            }),
                        })
                        .collect(),
                }),
                Any::Bytes(v) => Value::BytesValue(v),
            }
        }
    }

    fn attributes_to_key_value(attributes: BTreeMap<Cow<'static, str>, Any>) -> Vec<KeyValue> {
        attributes
            .into_iter()
            .map(|(key, value)| KeyValue {
                key: key.into_owned(),
                value: Some(AnyValue {
                    value: Some(value.into()),
                }),
            })
            .collect()
    }

    impl From<opentelemetry::sdk::logs::LogRecord> for LogRecord {
        fn from(log_record: opentelemetry::sdk::logs::LogRecord) -> Self {
            let trace_context = log_record.trace_context.as_ref();
            let severity_number = match log_record.severity_number {
                Some(Severity::Trace) => SeverityNumber::Trace,
                Some(Severity::Trace2) => SeverityNumber::Trace2,
                Some(Severity::Trace3) => SeverityNumber::Trace3,
                Some(Severity::Trace4) => SeverityNumber::Trace4,
                Some(Severity::Debug) => SeverityNumber::Debug,
                Some(Severity::Debug2) => SeverityNumber::Debug2,
                Some(Severity::Debug3) => SeverityNumber::Debug3,
                Some(Severity::Debug4) => SeverityNumber::Debug4,
                Some(Severity::Info) => SeverityNumber::Info,
                Some(Severity::Info2) => SeverityNumber::Info2,
                Some(Severity::Info3) => SeverityNumber::Info3,
                Some(Severity::Info4) => SeverityNumber::Info4,
                Some(Severity::Warn) => SeverityNumber::Warn,
                Some(Severity::Warn2) => SeverityNumber::Warn2,
                Some(Severity::Warn3) => SeverityNumber::Warn3,
                Some(Severity::Warn4) => SeverityNumber::Warn4,
                Some(Severity::Error) => SeverityNumber::Error,
                Some(Severity::Error2) => SeverityNumber::Error2,
                Some(Severity::Error3) => SeverityNumber::Error3,
                Some(Severity::Error4) => SeverityNumber::Error4,
                Some(Severity::Fatal) => SeverityNumber::Fatal,
                Some(Severity::Fatal2) => SeverityNumber::Fatal2,
                Some(Severity::Fatal3) => SeverityNumber::Fatal3,
                Some(Severity::Fatal4) => SeverityNumber::Fatal4,
                None => SeverityNumber::Unspecified
            };

            let record = LogRecord {
                time_unix_nano: log_record.timestamp.map(to_nanos).unwrap_or_default(),
                observed_time_unix_nano: log_record
                    .observed_timestamp
                    .map(to_nanos)
                    .unwrap_or_default(),
                severity_number: severity_number.into(),
                severity_text: log_record.severity_text.map(Into::into).unwrap_or_default(),
                body: Some(AnyValue {
                    value: log_record.body.map(Into::into),
                }),
                attributes: log_record
                    .attributes
                    .map(attributes_to_keyvalue)
                    .unwrap_or_default(),
                dropped_attributes_count: 0,
                flags: trace_context
                    .map(|ctx| {
                        ctx.trace_flags
                            .map(|flags| flags.to_u8() as u32)
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                span_id: trace_context
                    .map(|ctx| ctx.span_id.to_bytes().to_vec())
                    .unwrap_or_default(),
                trace_id: trace_context
                    .map(|ctx| ctx.trace_id.to_bytes().to_vec())
                    .unwrap_or_default(),
                ..Default::default()
            };
            record
        }
    }

    impl From<opentelemetry::sdk::export::logs::LogData> for ResourceLogs {
        fn from(log_data: opentelemetry::sdk::export::logs::LogData) -> Self {
            ResourceLogs {
                resource: Some(Resource {
                    attributes: resource_attributes(log_data.resource.as_ref().map(AsRef::as_ref))
                        .0,
                    dropped_attributes_count: 0,
                }),
                schema_url: "".to_string(),
                instrumentation_library_logs: vec![InstrumentationLibraryLogs {
                    schema_url: log_data
                        .instrumentation
                        .schema_url
                        .clone()
                        .map(Into::into)
                        .unwrap_or_default(),
                    instrumentation_library: Some(log_data.instrumentation.into()),
                    log_records: vec![log_data.record.into()],
                }],
            }
        }
    }
}

#[cfg(feature = "gen-protoc")]
pub mod grpcio {
    use std::collections::BTreeMap;

    use crate::{
        proto::grpcio::{
            common::{AnyValue, AnyValue_oneof_value, ArrayValue, KeyValue, KeyValueList},
            logs::{InstrumentationLibraryLogs, LogRecord, ResourceLogs, SeverityNumber},
            resource::Resource,
        },
        transform::common::grpcio::resource_attributes,
    };
    use opentelemetry::sdk::logs::{Any, Severity};
    use protobuf::{RepeatedField, SingularPtrField};

    use super::*;

    impl From<Any> for AnyValue_oneof_value {
        fn from(value: Any) -> Self {
            match value {
                Any::Double(f) => AnyValue_oneof_value::double_value(f),
                Any::Int(i) => AnyValue_oneof_value::int_value(i),
                Any::String(s) => AnyValue_oneof_value::string_value(s.into()),
                Any::Boolean(b) => AnyValue_oneof_value::bool_value(b),
                Any::ListAny(v) => AnyValue_oneof_value::array_value(ArrayValue {
                    values: RepeatedField::from_vec(
                        v.into_iter()
                            .map(|v| AnyValue {
                                value: Some(v.into()),
                                ..Default::default()
                            })
                            .collect(),
                    ),
                    ..Default::default()
                }),
                Any::Map(m) => AnyValue_oneof_value::kvlist_value(KeyValueList {
                    values: RepeatedField::from_vec(
                        m.into_iter()
                            .map(|(key, value)| KeyValue {
                                key: key.into_owned(),
                                value: SingularPtrField::some(AnyValue {
                                    value: Some(value.into()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })
                            .collect(),
                    ),
                    ..Default::default()
                }),
                Any::Bytes(v) => AnyValue_oneof_value::bytes_value(v),
            }
        }
    }

    fn attributes_to_keyvalue(attributes: BTreeMap<Cow<'static, str>, Any>) -> Vec<KeyValue> {
        attributes
            .into_iter()
            .map(|(key, value)| KeyValue {
                key: key.into_owned(),
                value: SingularPtrField::some(AnyValue {
                    value: Some(value.into()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect()
    }

    impl From<opentelemetry::sdk::logs::LogRecord> for LogRecord {
        fn from(log_record: opentelemetry::sdk::logs::LogRecord) -> Self {
            let trace_context = log_record.trace_context.as_ref();
            let severity_number = match log_record.severity_number {
                Some(Severity::Trace) => SeverityNumber::SEVERITY_NUMBER_TRACE,
                Some(Severity::Trace2) => SeverityNumber::SEVERITY_NUMBER_TRACE2,
                Some(Severity::Trace3) => SeverityNumber::SEVERITY_NUMBER_TRACE3,
                Some(Severity::Trace4) => SeverityNumber::SEVERITY_NUMBER_TRACE4,
                Some(Severity::Debug) => SeverityNumber::SEVERITY_NUMBER_DEBUG,
                Some(Severity::Debug2) => SeverityNumber::SEVERITY_NUMBER_DEBUG2,
                Some(Severity::Debug3) => SeverityNumber::SEVERITY_NUMBER_DEBUG3,
                Some(Severity::Debug4) => SeverityNumber::SEVERITY_NUMBER_DEBUG4,
                Some(Severity::Info) => SeverityNumber::SEVERITY_NUMBER_INFO,
                Some(Severity::Info2) => SeverityNumber::SEVERITY_NUMBER_INFO2,
                Some(Severity::Info3) => SeverityNumber::SEVERITY_NUMBER_INFO3,
                Some(Severity::Info4) => SeverityNumber::SEVERITY_NUMBER_INFO4,
                Some(Severity::Warn) => SeverityNumber::SEVERITY_NUMBER_WARN,
                Some(Severity::Warn2) => SeverityNumber::SEVERITY_NUMBER_WARN2,
                Some(Severity::Warn3) => SeverityNumber::SEVERITY_NUMBER_WARN3,
                Some(Severity::Warn4) => SeverityNumber::SEVERITY_NUMBER_WARN4,
                Some(Severity::Error) => SeverityNumber::SEVERITY_NUMBER_ERROR,
                Some(Severity::Error2) => SeverityNumber::SEVERITY_NUMBER_ERROR2,
                Some(Severity::Error3) => SeverityNumber::SEVERITY_NUMBER_ERROR3,
                Some(Severity::Error4) => SeverityNumber::SEVERITY_NUMBER_ERROR4,
                Some(Severity::Fatal) => SeverityNumber::SEVERITY_NUMBER_FATAL,
                Some(Severity::Fatal2) => SeverityNumber::SEVERITY_NUMBER_FATAL2,
                Some(Severity::Fatal3) => SeverityNumber::SEVERITY_NUMBER_FATAL3,
                Some(Severity::Fatal4) => SeverityNumber::SEVERITY_NUMBER_FATAL4,
                None => SeverityNumber::SEVERITY_NUMBER_UNSPECIFIED,
            };

            LogRecord {
                time_unix_nano: log_record.timestamp.map(to_nanos).unwrap_or(0),
                severity_number,
                severity_text: log_record.severity_text.map(Into::into).unwrap_or_default(),
                body: SingularPtrField::some(AnyValue {
                    value: log_record.body.map(Into::into),
                    ..Default::default()
                }),
                attributes: RepeatedField::from_vec(
                    log_record
                        .attributes
                        .map(attributes_to_keyvalue)
                        .unwrap_or_default(),
                ),
                dropped_attributes_count: 0,
                flags: trace_context
                    .map(|ctx| {
                        ctx.trace_flags
                            .map(|flags| flags.to_u8() as u32)
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                span_id: trace_context
                    .map(|ctx| ctx.span_id.to_bytes().to_vec())
                    .unwrap_or_default(),
                trace_id: trace_context
                    .map(|ctx| ctx.trace_id.to_bytes().to_vec())
                    .unwrap_or_default(),
                ..Default::default()
            }
        }
    }

    impl From<opentelemetry::sdk::export::logs::LogData> for ResourceLogs {
        fn from(log_data: opentelemetry::sdk::export::logs::LogData) -> Self {
            ResourceLogs {
                resource: SingularPtrField::some(Resource {
                    attributes: resource_attributes(log_data.resource.as_ref().map(AsRef::as_ref))
                        .0,
                    dropped_attributes_count: 0,
                    ..Default::default()
                }),
                schema_url: "".to_string(),
                instrumentation_library_logs: RepeatedField::from_vec(vec![
                    InstrumentationLibraryLogs {
                        schema_url: log_data
                            .instrumentation
                            .schema_url
                            .clone()
                            .map(Into::into)
                            .unwrap_or_default(),
                        instrumentation_library: SingularPtrField::some(
                            log_data.instrumentation.into(),
                        ),
                        log_records: RepeatedField::from_vec(vec![log_data.record.into()]),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }
        }
    }
}
