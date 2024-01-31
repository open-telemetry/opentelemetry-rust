use log::{Level, Metadata, Record};
use opentelemetry::logs::{AnyValue, LogRecordBuilder, Logger, LoggerProvider, Severity};
use std::borrow::Cow;

pub struct OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    logger: L,
    _phantom: std::marker::PhantomData<P>, // P is not used in this struct
}

impl<P, L> log::Log for OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    fn enabled(&self, _metadata: &Metadata) -> bool {
        #[cfg(feature = "logs_level_enabled")]
        return self
            .logger
            .event_enabled(severity_of_level(_metadata.level()), _metadata.target());
        #[cfg(not(feature = "logs_level_enabled"))]
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.logger.emit(
                LogRecordBuilder::new()
                    .with_severity_number(severity_of_level(record.level()))
                    .with_severity_text(record.level().as_str())
                    // Not populating ObservedTimestamp, instead relying on OpenTelemetry
                    // API to populate it with current time.
                    .with_body(AnyValue::from(record.args().to_string()))
                    .build(),
            );
        }
    }

    fn flush(&self) {}
}

impl<P, L> OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    pub fn new(provider: &P) -> Self {
        OpenTelemetryLogBridge {
            logger: provider.versioned_logger(
                "opentelemetry-log-appender",
                Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
                None,
                None,
            ),
            _phantom: Default::default(),
        }
    }
}

const fn severity_of_level(level: Level) -> Severity {
    match level {
        Level::Error => Severity::Error,
        Level::Warn => Severity::Warn,
        Level::Info => Severity::Info,
        Level::Debug => Severity::Debug,
        Level::Trace => Severity::Trace,
    }
}

#[cfg(all(test, feature = "testing", feature = "logs"))]
mod tests {
    use super::OpenTelemetryLogBridge;

    use opentelemetry_sdk::{logs::LoggerProvider, testing::logs::InMemoryLogsExporter};

    use log::{Level, Log};

    #[test]
    fn logbridge_with_default_metadata_is_enabled() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter)
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        // As a result of using `with_simple_exporter` while building the logger provider,
        // the processor used is a `SimpleLogProcessor` which has an implementation of `event_enabled`
        // that always returns true.
        #[cfg(feature = "logs_level_enabled")]
        assert_eq!(
            otel_log_appender.enabled(&log::Metadata::builder().build()),
            true
        );
        #[cfg(not(feature = "logs_level_enabled"))]
        assert_eq!(
            otel_log_appender.enabled(&log::Metadata::builder().build()),
            true
        );
    }

    #[test]
    fn logbridge_with_record_can_log() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);

        log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
        log::set_max_level(Level::Trace.to_level_filter());

        log::trace!("TRACE");
        log::debug!("DEBUG");
        log::info!("INFO");
        log::warn!("WARN");
        log::error!("ERROR");

        let logs = exporter.get_emitted_logs().unwrap();

        assert_eq!(logs.len(), 5);
        for log in logs {
            let body: String = match log.record.body.as_ref().unwrap() {
                super::AnyValue::String(s) => s.to_string(),
                _ => panic!("AnyValue::String expected"),
            };
            assert_eq!(body, log.record.severity_text.unwrap());
        }
    }

    #[test]
    fn test_flush() {
        let exporter = InMemoryLogsExporter::default();

        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter)
            .build();

        let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
        otel_log_appender.flush();
    }

    #[test]
    fn check_level_severities() {
        assert_eq!(
            super::severity_of_level(log::Level::Error),
            opentelemetry::logs::Severity::Error
        );
        assert_eq!(
            super::severity_of_level(log::Level::Warn),
            opentelemetry::logs::Severity::Warn
        );
        assert_eq!(
            super::severity_of_level(log::Level::Info),
            opentelemetry::logs::Severity::Info
        );
        assert_eq!(
            super::severity_of_level(log::Level::Debug),
            opentelemetry::logs::Severity::Debug
        );
        assert_eq!(
            super::severity_of_level(log::Level::Trace),
            opentelemetry::logs::Severity::Trace
        );
    }
}
