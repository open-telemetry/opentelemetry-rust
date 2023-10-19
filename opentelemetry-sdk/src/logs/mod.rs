//! # OpenTelemetry Log SDK

mod config;
mod log_emitter;
mod log_processor;

pub use config::{config, Config};
pub use log_emitter::{Builder, Logger, LoggerProvider};
pub use log_processor::{
    BatchConfig, BatchLogProcessor, BatchLogProcessorBuilder, BatchMessage, LogProcessor,
    SimpleLogProcessor,
};

#[cfg(all(test, feature = "testing"))]
mod tests {
    // use std::thread::sleep;

    use super::*;
    use crate::testing::logs::InMemoryLogsExporter;
    use opentelemetry::{logs::AnyValue, Key};
    use opentelemetry_appender_tracing::layer;
    use tracing::error;
    use tracing_subscriber::prelude::*;

    #[test]
    fn logging_sdk_test() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_log_processor(SimpleLogProcessor::new(Box::new(exporter.clone())))
            .build();

        // Act
        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        tracing_subscriber::registry().with(layer).init();
        error!(target: "my-system", event_id = 20, event_name = "my-event_name");

        logger_provider.force_flush();
        // TODO: To remove this comment.
        // The test will fail without the sleep prior to the flush fix.
        // sleep(std::time::Duration::from_millis(10));

        // Assert
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs.get(0).expect("Atleast one log is expected to be present.");
        let attributes: Vec<(Key, AnyValue)> = log.record.attributes.clone().expect("Attributes are expected");
        assert_eq!(attributes.len(), 2);
    }
}
