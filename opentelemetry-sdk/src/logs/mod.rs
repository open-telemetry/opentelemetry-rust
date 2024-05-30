//! # OpenTelemetry Log SDK

mod log_emitter;
mod log_processor;
mod record;

pub use log_emitter::{Builder, Logger, LoggerProvider};
pub use log_processor::{
    BatchConfig, BatchConfigBuilder, BatchLogProcessor, BatchLogProcessorBuilder, LogProcessor,
    SimpleLogProcessor,
};
pub use record::{LogRecord, TraceContext};

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::logs::InMemoryLogsExporter;
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
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_resource(resource.clone())
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let logger = logger_provider.logger("test-logger");
        let mut log_record = logger.create_log_record();
        log_record.set_severity_number(Severity::Error);
        log_record.set_severity_text("Error".into());

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
        assert_eq!(log.instrumentation.name, "test-logger");
        assert_eq!(log.record.severity_number, Some(Severity::Error));
        let attributes: Vec<(Key, AnyValue)> = log
            .record
            .attributes
            .clone()
            .expect("Attributes are expected");
        assert_eq!(attributes.len(), 10);
        for i in 1..=10 {
            assert!(log.record.attributes.clone().unwrap().contains(&(
                Key::new(format!("key{}", i)),
                AnyValue::String(format!("value{}", i).into())
            )));
        }

        // validate Resource
        assert_eq!(&resource, log.resource.borrow());
    }

    #[test]
    fn logger_attributes() {
        let provider = LoggerProvider::builder().build();
        let logger = provider
            .logger_builder("test_logger")
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .with_attributes(vec![(KeyValue::new("test_k", "test_v"))])
            .build();
        let instrumentation_library = logger.instrumentation_library();
        let attributes = &instrumentation_library.attributes;
        assert_eq!(instrumentation_library.name, "test_logger");
        assert_eq!(
            instrumentation_library.schema_url,
            Some("https://opentelemetry.io/schema/1.0.0".into())
        );
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "test_k".into());
        assert_eq!(attributes[0].value, "test_v".into());
    }

    #[test]
    #[allow(deprecated)]
    fn versioned_logger_options() {
        let provider = LoggerProvider::builder().build();
        let logger = provider.versioned_logger(
            "test_logger",
            Some("v1.2.3".into()),
            Some("https://opentelemetry.io/schema/1.0.0".into()),
            Some(vec![(KeyValue::new("test_k", "test_v"))]),
        );
        let instrumentation_library = logger.instrumentation_library();
        let attributes = &instrumentation_library.attributes;
        assert_eq!(instrumentation_library.version, Some("v1.2.3".into()));
        assert_eq!(
            instrumentation_library.schema_url,
            Some("https://opentelemetry.io/schema/1.0.0".into())
        );
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "test_k".into());
        assert_eq!(attributes[0].value, "test_v".into());
    }
}
