#[cfg(feature = "gen-tonic")]
pub mod tonic {
    use crate::{
        tonic::{
            common::v1::{any_value::Value, AnyValue, ArrayValue, KeyValue, KeyValueList},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
            Attributes,
        },
        transform::common::{to_nanos, tonic::resource_attributes},
    };
    use opentelemetry::logs::{AnyValue as LogsAnyValue, Severity};

    impl From<LogsAnyValue> for AnyValue {
        fn from(value: LogsAnyValue) -> Self {
            AnyValue {
                value: Some(value.into()),
            }
        }
    }

    impl From<LogsAnyValue> for Value {
        fn from(value: LogsAnyValue) -> Self {
            match value {
                LogsAnyValue::Double(f) => Value::DoubleValue(f),
                LogsAnyValue::Int(i) => Value::IntValue(i),
                LogsAnyValue::String(s) => Value::StringValue(s.into()),
                LogsAnyValue::Boolean(b) => Value::BoolValue(b),
                LogsAnyValue::ListAny(v) => Value::ArrayValue(ArrayValue {
                    values: v
                        .into_iter()
                        .map(|v| AnyValue {
                            value: Some(v.into()),
                        })
                        .collect(),
                }),
                LogsAnyValue::Map(m) => Value::KvlistValue(KeyValueList {
                    values: m
                        .into_iter()
                        .map(|(key, value)| KeyValue {
                            key: key.into(),
                            value: Some(AnyValue {
                                value: Some(value.into()),
                            }),
                        })
                        .collect(),
                }),
                LogsAnyValue::Bytes(v) => Value::BytesValue(v),
            }
        }
    }

    impl From<opentelemetry::logs::LogRecord> for LogRecord {
        fn from(log_record: opentelemetry::logs::LogRecord) -> Self {
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
                None => SeverityNumber::Unspecified,
            };

            LogRecord {
                time_unix_nano: log_record.timestamp.map(to_nanos).unwrap_or_default(),
                observed_time_unix_nano: to_nanos(log_record.observed_timestamp),
                severity_number: severity_number.into(),
                severity_text: log_record.severity_text.map(Into::into).unwrap_or_default(),
                body: log_record.body.map(Into::into),
                attributes: log_record
                    .attributes
                    .map(Attributes::from_iter)
                    .unwrap_or_default()
                    .0,
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
            }
        }
    }

    impl From<opentelemetry_sdk::export::logs::LogData> for ResourceLogs {
        fn from(log_data: opentelemetry_sdk::export::logs::LogData) -> Self {
            ResourceLogs {
                resource: Some(Resource {
                    attributes: resource_attributes(&log_data.resource).0,
                    dropped_attributes_count: 0,
                }),
                schema_url: log_data
                    .resource
                    .schema_url()
                    .map(Into::into)
                    .unwrap_or_default(),
                scope_logs: vec![ScopeLogs {
                    schema_url: log_data
                        .instrumentation
                        .schema_url
                        .clone()
                        .map(Into::into)
                        .unwrap_or_default(),
                    scope: Some(log_data.instrumentation.into()),
                    log_records: vec![log_data.record.into()],
                }],
            }
        }
    }
}

#[cfg(feature = "gen-grpcio")]
pub mod grpcio {
    use crate::{
        grpcio::{
            common::v1::{any_value::Value, AnyValue, ArrayValue, KeyValue, KeyValueList},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
            Attributes,
        },
        transform::common::{grpcio::resource_attributes, to_nanos},
    };
    use opentelemetry::logs::{AnyValue as LogsAnyValue, Severity};

    impl From<LogsAnyValue> for AnyValue {
        fn from(value: LogsAnyValue) -> Self {
            AnyValue {
                value: Some(value.into()),
            }
        }
    }

    impl From<LogsAnyValue> for Value {
        fn from(value: LogsAnyValue) -> Self {
            match value {
                LogsAnyValue::Double(f) => Value::DoubleValue(f),
                LogsAnyValue::Int(i) => Value::IntValue(i),
                LogsAnyValue::String(s) => Value::StringValue(s.into()),
                LogsAnyValue::Boolean(b) => Value::BoolValue(b),
                LogsAnyValue::ListAny(v) => Value::ArrayValue(ArrayValue {
                    values: v
                        .into_iter()
                        .map(|v| AnyValue {
                            value: Some(v.into()),
                        })
                        .collect(),
                }),
                LogsAnyValue::Map(m) => Value::KvlistValue(KeyValueList {
                    values: m
                        .into_iter()
                        .map(|(key, value)| KeyValue {
                            key: key.into(),
                            value: Some(AnyValue {
                                value: Some(value.into()),
                            }),
                        })
                        .collect(),
                }),
                LogsAnyValue::Bytes(v) => Value::BytesValue(v),
            }
        }
    }

    impl From<opentelemetry::logs::LogRecord> for LogRecord {
        fn from(log_record: opentelemetry::logs::LogRecord) -> Self {
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
                None => SeverityNumber::Unspecified,
            };

            LogRecord {
                time_unix_nano: log_record.timestamp.map(to_nanos).unwrap_or_default(),
                observed_time_unix_nano: to_nanos(log_record.observed_timestamp),
                severity_number: severity_number.into(),
                severity_text: log_record.severity_text.map(Into::into).unwrap_or_default(),
                body: log_record.body.map(Into::into),
                attributes: log_record
                    .attributes
                    .map(Attributes::from_iter)
                    .unwrap_or_default()
                    .0,
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
            }
        }
    }

    impl From<opentelemetry_sdk::export::logs::LogData> for ResourceLogs {
        fn from(log_data: opentelemetry_sdk::export::logs::LogData) -> Self {
            ResourceLogs {
                resource: Some(Resource {
                    attributes: resource_attributes(&log_data.resource).0,
                    dropped_attributes_count: 0,
                }),
                schema_url: log_data
                    .resource
                    .schema_url()
                    .map(Into::into)
                    .unwrap_or_default(),
                scope_logs: vec![ScopeLogs {
                    schema_url: log_data
                        .instrumentation
                        .schema_url
                        .clone()
                        .map(Into::into)
                        .unwrap_or_default(),
                    scope: Some(log_data.instrumentation.into()),
                    log_records: vec![log_data.record.into()],
                }],
            }
        }
    }
}
