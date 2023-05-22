use log::{Level, Record, Metadata};
use opentelemetry_api::logs::{LoggerProvider as _, Logger as _, LogRecordBuilder, Severity, AnyValue};
use opentelemetry_sdk::logs::{LoggerProvider, Logger};

pub struct OpenTelemetryLogBridge<>{
    logger: Logger,
    min_level: Level,
}

impl log::Log for OpenTelemetryLogBridge {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.logger.emit(LogRecordBuilder::new()
            .with_severity_number(map_severity_to_otel_severity(record.level()))
            .with_severity_text(record.level().as_str())
            .with_body(AnyValue::from(record.args().to_string()))
            .build());
        }
    }

    fn flush(&self) {}
}

impl OpenTelemetryLogBridge {
    pub fn new(level: Level, provider: &LoggerProvider) -> Self {        
            log::set_max_level(level.to_level_filter());
            OpenTelemetryLogBridge {
                logger: provider.logger("opentelemetry-log-appender"),
                min_level: level,
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

