use log::{Level, Metadata, Record};
use opentelemetry_api::logs::{AnyValue, LogRecordBuilder, Logger, LoggerProvider, Severity};

pub struct OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    logger: L,
    min_level: Level,
    _phantom: std::marker::PhantomData<P>, // P is not used in this struct
}

impl<P, L> log::Log for OpenTelemetryLogBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.logger.emit(
                LogRecordBuilder::new()
                    .with_severity_number(map_severity_to_otel_severity(record.level()))
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
    pub fn new(level: Level, provider: &P) -> Self {
        log::set_max_level(level.to_level_filter());
        OpenTelemetryLogBridge {
            logger: provider.logger("opentelemetry-log-appender"),
            min_level: level,
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
