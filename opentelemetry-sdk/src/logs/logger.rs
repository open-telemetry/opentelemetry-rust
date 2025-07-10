use super::{SdkLogRecord, SdkLoggerProvider, TraceContext};
use opentelemetry::{trace::TraceContextExt, Context, InstrumentationScope};

#[cfg(feature = "spec_unstable_logs_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::time::now;

#[derive(Debug, Clone)]
/// The object for emitting [`LogRecord`]s.
///
/// [`LogRecord`]: opentelemetry::logs::LogRecord
pub struct SdkLogger {
    scope: InstrumentationScope,
    provider: SdkLoggerProvider,
}

impl SdkLogger {
    pub(crate) fn new(scope: InstrumentationScope, provider: SdkLoggerProvider) -> Self {
        SdkLogger { scope, provider }
    }
}

impl opentelemetry::logs::Logger for SdkLogger {
    type LogRecord = SdkLogRecord;

    fn create_log_record(&self) -> Self::LogRecord {
        SdkLogRecord::new()
    }

    /// Emit a `LogRecord`.
    fn emit(&self, mut record: Self::LogRecord) {
        if Context::is_current_telemetry_suppressed() {
            return;
        }
        let provider = &self.provider;
        let processors = provider.log_processors();

        //let mut log_record = record;
        if record.trace_context.is_none() {
            Context::map_current(|cx| {
                cx.has_active_span().then(|| {
                    record.trace_context = Some(TraceContext::from(cx.span().span_context()))
                })
            });
        }
        if record.observed_timestamp.is_none() {
            record.observed_timestamp = Some(now());
        }

        for p in processors {
            p.emit(&mut record, &self.scope);
        }
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    #[inline]
    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        if Context::is_current_telemetry_suppressed() {
            return false;
        }
        self.provider
            .log_processors()
            .iter()
            .any(|processor| processor.event_enabled(level, target, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::SdkLoggerProvider;
    use opentelemetry::InstrumentationScope;

    #[test]
    fn test_sdk_logger_clone() {
        let scope = InstrumentationScope::builder("test").build();
        let provider = SdkLoggerProvider::builder().build();
        let logger = SdkLogger::new(scope, provider);

        // Test that clone works - this is the main goal
        #[allow(clippy::redundant_clone)]
        let cloned_logger = logger.clone();

        // Verify that both loggers are valid by checking they have the same scope name
        assert_eq!(logger.scope.name(), cloned_logger.scope.name());
    }
}
