//! # OpenTelemetry Log SDK
mod error;
mod export;
mod log_processor;
mod logger;
mod logger_provider;
pub(crate) mod record;

/// In-Memory log exporter for testing purpose.
#[cfg(any(feature = "testing", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "testing", test))))]
pub mod in_memory_exporter;
#[cfg(any(feature = "testing", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "testing", test))))]
pub use in_memory_exporter::{InMemoryLogExporter, InMemoryLogExporterBuilder};

pub use error::{LogError, LogResult};
pub use export::{LogBatch, LogExporter};
pub use log_processor::{
    BatchConfig, BatchConfigBuilder, BatchLogProcessor, BatchLogProcessorBuilder, LogProcessor,
    SimpleLogProcessor,
};
pub use logger::SdkLogger;
pub use logger_provider::{LoggerProviderBuilder, SdkLoggerProvider};
pub use record::{SdkLogRecord, TraceContext};

#[cfg(feature = "experimental_logs_batch_log_processor_with_async_runtime")]
/// Module for BatchLogProcessor with async runtime.
pub mod log_processor_with_async_runtime;

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::Resource;
    use opentelemetry::logs::LogRecord;
    use opentelemetry::logs::{Logger, LoggerProvider, Severity};
    use opentelemetry::InstrumentationScope;
    use opentelemetry::{logs::AnyValue, Key, KeyValue};
    use std::borrow::Borrow;
    use std::collections::HashMap;

    #[test]
    fn logging_sdk_test() {
        // Arrange
        let resource = Resource::builder_empty()
            .with_attributes([
                KeyValue::new("k1", "v1"),
                KeyValue::new("k2", "v2"),
                KeyValue::new("k3", "v3"),
                KeyValue::new("k4", "v4"),
            ])
            .build();
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource.clone())
            .with_log_processor(SimpleLogProcessor::new(exporter.clone()))
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
        let provider = SdkLoggerProvider::builder().build();
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
