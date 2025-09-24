#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use crate::{
        tonic::{
            common::v1::{
                any_value::Value, AnyValue, ArrayValue, InstrumentationScope, KeyValue,
                KeyValueList,
            },
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
        },
        transform::common::{to_nanos, tonic::ResourceAttributesWithSchema},
    };
    use opentelemetry::logs::{AnyValue as LogsAnyValue, Severity};
    use opentelemetry_sdk::logs::LogBatch;
    use std::borrow::Cow;
    use std::collections::HashMap;

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
                LogsAnyValue::Bytes(v) => Value::BytesValue(*v),
                _ => unreachable!("Nonexistent value type"),
            }
        }
    }

    impl From<&opentelemetry_sdk::logs::SdkLogRecord> for LogRecord {
        fn from(log_record: &opentelemetry_sdk::logs::SdkLogRecord) -> Self {
            let trace_context = log_record.trace_context();
            let severity_number = match log_record.severity_number() {
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
                time_unix_nano: log_record.timestamp().map(to_nanos).unwrap_or_default(),
                observed_time_unix_nano: to_nanos(log_record.observed_timestamp().unwrap()),
                attributes: {
                    log_record
                        .attributes_iter()
                        .map(|kv| KeyValue {
                            key: kv.0.to_string(),
                            value: Some(AnyValue {
                                value: Some(kv.1.clone().into()),
                            }),
                        })
                        .collect()
                },
                event_name: log_record.event_name().unwrap_or_default().into(),
                severity_number: severity_number.into(),
                severity_text: log_record
                    .severity_text()
                    .map(Into::into)
                    .unwrap_or_default(),
                body: log_record.body().cloned().map(Into::into),
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

    impl
        From<(
            (
                &opentelemetry_sdk::logs::SdkLogRecord,
                &opentelemetry::InstrumentationScope,
            ),
            &ResourceAttributesWithSchema,
        )> for ResourceLogs
    {
        fn from(
            data: (
                (
                    &opentelemetry_sdk::logs::SdkLogRecord,
                    &opentelemetry::InstrumentationScope,
                ),
                &ResourceAttributesWithSchema,
            ),
        ) -> Self {
            let ((log_record, instrumentation), resource) = data;

            ResourceLogs {
                resource: Some(Resource {
                    attributes: resource.attributes.0.clone(),
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                schema_url: resource.schema_url.clone().unwrap_or_default(),
                scope_logs: vec![ScopeLogs {
                    schema_url: instrumentation
                        .schema_url()
                        .map(ToOwned::to_owned)
                        .unwrap_or_default(),
                    scope: Some((instrumentation, log_record.target().cloned()).into()),
                    log_records: vec![log_record.into()],
                }],
            }
        }
    }

    pub fn group_logs_by_resource_and_scope(
        logs: LogBatch<'_>,
        resource: &ResourceAttributesWithSchema,
    ) -> Vec<ResourceLogs> {
        // Group logs by target or instrumentation name
        let scope_map = logs.iter().fold(
            HashMap::new(),
            |mut scope_map: HashMap<
                Cow<'static, str>,
                Vec<(
                    &opentelemetry_sdk::logs::SdkLogRecord,
                    &opentelemetry::InstrumentationScope,
                )>,
            >,
             (log_record, instrumentation)| {
                let key = log_record
                    .target()
                    .cloned()
                    .unwrap_or_else(|| Cow::Owned(instrumentation.name().to_owned()));
                scope_map
                    .entry(key)
                    .or_default()
                    .push((log_record, instrumentation));
                scope_map
            },
        );

        let scope_logs = scope_map
            .into_iter()
            .map(|(key, log_data)| ScopeLogs {
                scope: Some(new_instrumentation_scope(log_data.first().unwrap().1, key)),
                schema_url: resource.schema_url.clone().unwrap_or_default(),
                log_records: log_data
                    .into_iter()
                    .map(|(log_record, _)| log_record.into())
                    .collect(),
            })
            .collect();

        vec![ResourceLogs {
            resource: Some(Resource {
                attributes: resource.attributes.0.clone(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_logs,
            schema_url: resource.schema_url.clone().unwrap_or_default(),
        }]
    }

    /// Ensure that InstrumentationScope is propagated correctly.
    /// These are the rules:
    ///
    /// 1. If the log scope name matches the key, return a new scope with all the fields from the original scope.
    /// 2. Otherwise, return a new scope with the key as the name of the scope, and the rest of the fields empty.
    fn new_instrumentation_scope(
        log_scope: &opentelemetry::InstrumentationScope,
        key: Cow<'static, str>,
    ) -> InstrumentationScope {
        if key == log_scope.name() {
            return InstrumentationScope::from((log_scope, None));
        }

        InstrumentationScope::from((log_scope, Some(key)))
    }
}

#[cfg(test)]
mod tests {
    use crate::transform::common::tonic::ResourceAttributesWithSchema;
    use opentelemetry::logs::LogRecord as _;
    use opentelemetry::logs::Logger;
    use opentelemetry::logs::LoggerProvider;
    use opentelemetry::time::now;
    use opentelemetry::InstrumentationScope;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::error::OTelSdkResult;
    use opentelemetry_sdk::logs::LogProcessor;
    use opentelemetry_sdk::logs::SdkLoggerProvider;
    use opentelemetry_sdk::{logs::LogBatch, logs::SdkLogRecord, Resource};

    #[derive(Debug)]
    struct MockProcessor;

    impl LogProcessor for MockProcessor {
        fn emit(&self, _record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {}

        fn force_flush(&self) -> OTelSdkResult {
            Ok(())
        }
    }

    fn create_test_log_data(
        instrumentation_name: &str,
        _message: &str,
        attrs: Vec<KeyValue>,
    ) -> (SdkLogRecord, InstrumentationScope) {
        let processor = MockProcessor {};
        let logger = SdkLoggerProvider::builder()
            .with_log_processor(processor)
            .build()
            .logger("test");
        let mut logrecord = logger.create_log_record();
        logrecord.set_timestamp(now());
        logrecord.set_observed_timestamp(now());
        let instrumentation = InstrumentationScope::builder(instrumentation_name.to_string())
            .with_attributes(attrs)
            .build();
        (logrecord, instrumentation)
    }

    #[test]
    fn test_group_logs_by_resource_and_scope_single_scope() {
        let resource = Resource::builder().build();
        let (log_record1, instrum_lib1) = create_test_log_data("test-lib", "Log 1", vec![]);
        let (log_record2, instrum_lib2) = create_test_log_data("test-lib", "Log 2", vec![]);

        let logs = [(&log_record1, &instrum_lib1), (&log_record2, &instrum_lib2)];
        let log_batch = LogBatch::new(&logs);
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema

        let grouped_logs =
            crate::transform::logs::tonic::group_logs_by_resource_and_scope(log_batch, &resource);

        assert_eq!(grouped_logs.len(), 1);
        let resource_logs = &grouped_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 1);

        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(scope_logs.log_records.len(), 2);
    }

    #[test]
    fn test_group_logs_by_resource_and_scope_multiple_scopes() {
        let resource = Resource::builder().build();
        let (log_record1, instrum_lib1) = create_test_log_data("lib1", "Log 1", vec![]);
        let (log_record2, instrum_lib2) = create_test_log_data("lib2", "Log 2", vec![]);

        let logs = [(&log_record1, &instrum_lib1), (&log_record2, &instrum_lib2)];
        let log_batch = LogBatch::new(&logs);
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema
        let grouped_logs =
            crate::transform::logs::tonic::group_logs_by_resource_and_scope(log_batch, &resource);

        assert_eq!(grouped_logs.len(), 1);
        let resource_logs = &grouped_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        let scope_logs_1 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib1")
            .unwrap();
        let scope_logs_2 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib2")
            .unwrap();

        assert_eq!(scope_logs_1.log_records.len(), 1);
        assert_eq!(scope_logs_2.log_records.len(), 1);
    }

    #[test]
    fn test_group_logs_by_resource_preserving_scope_attributes_when_log_target_empty() {
        let resource = Resource::builder().build();
        let (log_record1, instrum_lib1) =
            create_test_log_data("lib1", "Log 1", vec![KeyValue::new("key1", "value1")]);
        let (log_record2, instrum_lib2) =
            create_test_log_data("lib2", "Log 2", vec![KeyValue::new("key2", "value2")]);

        let logs = [(&log_record1, &instrum_lib1), (&log_record2, &instrum_lib2)];
        let log_batch = LogBatch::new(&logs);
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema
        let grouped_logs =
            crate::transform::logs::tonic::group_logs_by_resource_and_scope(log_batch, &resource);

        assert_eq!(grouped_logs.len(), 1);
        let resource_logs = &grouped_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        let scope_logs_1 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib1")
            .unwrap();
        let scope_logs_2 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib2")
            .unwrap();

        assert_eq!(1, scope_logs_1.scope.as_ref().unwrap().attributes.len());
        assert_eq!(1, scope_logs_2.scope.as_ref().unwrap().attributes.len());
    }

    #[test]
    fn test_group_logs_by_resource_preserving_scope_attributes_when_log_target_matching() {
        let resource = Resource::builder().build();
        let (mut log_record1, instrum_lib1) =
            create_test_log_data("lib1", "Log 1", vec![KeyValue::new("key1", "value1")]);
        let (mut log_record2, instrum_lib2) =
            create_test_log_data("lib2", "Log 2", vec![KeyValue::new("key2", "value2")]);

        let logs = [(&log_record1, &instrum_lib1), (&log_record2, &instrum_lib2)];
        let log_batch = LogBatch::new(&logs);
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema
        let grouped_logs =
            crate::transform::logs::tonic::group_logs_by_resource_and_scope(log_batch, &resource);

        // This makes the grouping to match the existent InstrumentationScope, preserving the scope attributes.
        log_record1.set_target("lib1");
        log_record2.set_target("lib2");

        assert_eq!(grouped_logs.len(), 1);
        let resource_logs = &grouped_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        let scope_logs_1 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib1")
            .unwrap();
        let scope_logs_2 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "lib2")
            .unwrap();

        assert_eq!(1, scope_logs_1.scope.as_ref().unwrap().attributes.len());
        assert_eq!(1, scope_logs_2.scope.as_ref().unwrap().attributes.len());
    }

    #[test]
    fn test_group_logs_by_resource_ignoring_scope_attributes_when_log_target_not_matching() {
        let resource = Resource::builder().build();
        let (mut log_record1, instrum_lib1) =
            create_test_log_data("lib1", "Log 1", vec![KeyValue::new("key1", "value1")]);
        let (mut log_record2, instrum_lib2) =
            create_test_log_data("lib2", "Log 2", vec![KeyValue::new("key2", "value2")]);

        // This makes the grouping to not match the existent InstrumentationScope, ignoring the scope attributes.
        log_record1.set_target("target1");
        log_record2.set_target("target2");

        let logs = [(&log_record1, &instrum_lib1), (&log_record2, &instrum_lib2)];
        let log_batch = LogBatch::new(&logs);
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema
        let grouped_logs =
            crate::transform::logs::tonic::group_logs_by_resource_and_scope(log_batch, &resource);

        assert_eq!(grouped_logs.len(), 1);
        let resource_logs = &grouped_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        let scope_logs_1 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "target1")
            .unwrap();
        let scope_logs_2 = &resource_logs
            .scope_logs
            .iter()
            .find(|scope| scope.scope.as_ref().unwrap().name == "target2")
            .unwrap();

        assert!(scope_logs_1.scope.as_ref().unwrap().attributes.is_empty());
        assert!(scope_logs_2.scope.as_ref().unwrap().attributes.is_empty());
    }
}
