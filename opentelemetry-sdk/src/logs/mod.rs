//! # OpenTelemetry Log SDK
mod error;
mod log_emitter;
mod log_processor;
pub(crate) mod record;

pub use error::{LogError, LogResult};
pub use log_emitter::{Builder, Logger, LoggerProvider};
pub use log_processor::{
    BatchConfig, BatchConfigBuilder, BatchLogProcessor, BatchLogProcessorBuilder, LogProcessor,
    SimpleLogProcessor,
};
use opentelemetry::InstrumentationScope;
pub use record::{LogRecord, TraceContext};

#[deprecated(
    since = "0.27.1",
    note = "The struct is not used anywhere in the SDK and will be removed in the next major release."
)]
/// `LogData` represents a single log event without resource context.
#[derive(Clone, Debug)]
pub struct LogData {
    /// Log record
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogEvent`.
    pub instrumentation: InstrumentationScope,
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::logs::InMemoryLogExporter;
    use crate::Resource;
    use opentelemetry::logs::LogRecord;
    use opentelemetry::logs::{Logger, LoggerProvider as _, Severity};
    use opentelemetry::{logs::AnyValue, Key, KeyValue};
    use std::borrow::Borrow;
    use std::collections::HashMap;

    #[test]
    fn logging_sdk_test() {
        // Arrange
        let resource = Resource::new(vec![
            KeyValue::new("k1", "v1"),
            KeyValue::new("k2", "v2"),
            KeyValue::new("k3", "v3"),
            KeyValue::new("k4", "v4"),
        ]);
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_resource(resource.clone())
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let logger = logger_provider.logger("test-logger");
        let mut log_record = logger.create_log_record();
        log_record.set_severity_number(Severity::Error);
        log_record.set_severity_text("Error");

        // Adding attributes using a vector with explicitly constructed Key and AnyValue objects.
        log_record.add_attributes(vec![
            (Key::new("key1"), AnyValue::from("value1")),
            (Key::new("key2"), AnyValue::from("value2")),
        ]);

        // Adding attributes using an array with explicitly constructed Key and AnyValue objects.
        log_record.add_attributes([
            (Key::new("key3"), AnyValue::from("value3")),
            (Key::new("key4"), AnyValue::from("value4")),
        ]);

        // Adding attributes using a vector with tuple auto-conversion to Key and AnyValue.
        log_record.add_attributes(vec![("key5", "value5"), ("key6", "value6")]);

        // Adding attributes using an array with tuple auto-conversion to Key and AnyValue.
        log_record.add_attributes([("key7", "value7"), ("key8", "value8")]);

        // Adding Attributes from a HashMap
        let mut attributes_map = HashMap::new();
        attributes_map.insert("key9", "value9");
        attributes_map.insert("key10", "value10");

        log_record.add_attributes(attributes_map);

        logger.emit(log_record);

        // Assert
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .first()
            .expect("Atleast one log is expected to be present.");
        assert_eq!(log.instrumentation.name(), "test-logger");
        assert_eq!(log.record.severity_number, Some(Severity::Error));
        assert_eq!(log.record.attributes_len(), 10);
        for i in 1..=10 {
            assert!(log.record.attributes_contains(
                &Key::new(format!("key{}", i)),
                &AnyValue::String(format!("value{}", i).into())
            ));
        }

        // validate Resource
        assert_eq!(&resource, log.resource.borrow());
    }

    #[test]
    #[allow(deprecated)]
    fn logger_attributes() {
        let provider = LoggerProvider::builder().build();
        let scope = InstrumentationScope::builder("test_logger")
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .with_attributes(vec![(KeyValue::new("test_k", "test_v"))])
            .build();

        let logger = provider.logger_with_scope(scope);
        let instrumentation_scope = logger.instrumentation_scope();
        assert_eq!(instrumentation_scope.name(), "test_logger");
        assert_eq!(
            instrumentation_scope.schema_url(),
            Some("https://opentelemetry.io/schema/1.0.0")
        );
        assert!(instrumentation_scope
            .attributes()
            .eq(&[KeyValue::new("test_k", "test_v")]));
    }
}
