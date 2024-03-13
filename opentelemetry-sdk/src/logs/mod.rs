//! # OpenTelemetry Log SDK

mod config;
mod log_emitter;
mod log_processor;

pub use config::{config, Config};
pub use log_emitter::{Builder, Logger, LoggerProvider};
pub use log_processor::{
    BatchConfig, BatchConfigBuilder, BatchLogProcessor, BatchLogProcessorBuilder, LogProcessor,
    SimpleLogProcessor,
};

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::logs::InMemoryLogsExporter;
    use opentelemetry::logs::{LogRecord, Logger, LoggerProvider as _, Severity};
    use opentelemetry::{logs::AnyValue, Key};
    use opentelemetry::global::{set_logger_provider, logger, shutdown_logger_provider};

    #[test]
    fn logging_sdk_test() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let logger = logger_provider.logger("test-logger");
        let mut log_record: LogRecord = LogRecord::default();
        log_record.severity_number = Some(Severity::Error);
        log_record.severity_text = Some("Error".into());
        let attributes = vec![
            (Key::new("key1"), "value1".into()),
            (Key::new("key2"), "value2".into()),
        ];
        log_record.attributes = Some(attributes);
        logger.emit(log_record);

        logger_provider.force_flush();

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
    fn logging_sdk_shutdown_test() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();
        set_logger_provider(logger_provider);

        // Act
        let logger = logger("test-logger");
        let mut log_record: LogRecord = LogRecord::default();
        log_record.severity_number = Some(Severity::Error);
        log_record.severity_text = Some("Error".into());
        let attributes = vec![
            (Key::new("key1"), "value1".into()),
            (Key::new("key2"), "value2".into()),
        ];
        log_record.attributes = Some(attributes);
        logger.emit(log_record);

        // Intentionally *not* calling shutdown/flush
        // on the provider, but instead relying on 
        // shutdown_logger_provider which causes
        // the global provider to be dropped, and
        // the sdk logger provider's drop implementation
        // will cause shutdown to be called on processors/exporters.
        shutdown_logger_provider();

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
}
