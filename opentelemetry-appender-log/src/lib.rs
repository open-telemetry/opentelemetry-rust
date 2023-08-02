use log::{Level, Metadata, Record};
use opentelemetry_api::logs::{AnyValue, LogRecordBuilder, Logger, LoggerProvider, Severity};
use std::time::SystemTime;

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
        return self.logger.event_enabled(
            map_severity_to_otel_severity(_metadata.level()),
            _metadata.target(),
        );
        #[cfg(not(feature = "logs_level_enabled"))]
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.logger.emit(
                LogRecordBuilder::new()
                    .with_severity_number(map_severity_to_otel_severity(record.level()))
                    .with_timestamp(SystemTime::now())
                    .with_severity_text(record.level().as_str())
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
            logger: provider.logger("opentelemetry-log-appender"),
            _phantom: Default::default(),
        }
    }
}

fn map_severity_to_otel_severity(level: Level) -> Severity {
    match level {
        Level::Error => Severity::Error,
        Level::Warn => Severity::Warn,
        Level::Info => Severity::Info,
        Level::Debug => Severity::Debug,
        Level::Trace => Severity::Trace,
    }
}
