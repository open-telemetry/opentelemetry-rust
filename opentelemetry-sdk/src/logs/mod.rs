//! # OpenTelemetry Log SDK

mod config;
mod log_emitter;
mod log_processor;
mod record;

pub use config::{config, Config};
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
    use opentelemetry::logs::LogRecord;
    use opentelemetry::logs::{Logger, LoggerProvider as _, Severity};
    use opentelemetry::{logs::AnyValue, Key, KeyValue};

    #[test]
    fn logging_sdk_test() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let logger = logger_provider.logger("test-logger");
        let mut log_record = logger.create_log_record();
        log_record.set_severity_number(Severity::Error);
        log_record.set_severity_text("Error".into());
        log_record.set_attributes(vec![
            (Key::new("key1"), "value1".into()),
            (Key::new("key2"), "value2".into()),
        ]);
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
        assert_eq!(attributes.len(), 2);
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
