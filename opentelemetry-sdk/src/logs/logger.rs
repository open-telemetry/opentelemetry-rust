use super::{SdkLogRecord, SdkLoggerProvider, TraceContext};
use opentelemetry::{trace::TraceContextExt, Context, InstrumentationScope};

#[cfg(feature = "spec_unstable_logs_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::time::now;

#[derive(Debug)]
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

    #[cfg(test)]
    pub(crate) fn instrumentation_scope(&self) -> &InstrumentationScope {
        &self.scope
    }
}

impl opentelemetry::logs::Logger for SdkLogger {
    type LogRecord = SdkLogRecord;

    fn create_log_record(&self) -> Self::LogRecord {
        SdkLogRecord::new()
    }

    /// Emit a `LogRecord`.
    fn emit(&self, mut record: Self::LogRecord) {
        let provider = &self.provider;
        let processors = provider.log_processors();

        //let mut log_record = record;
        if record.trace_context.is_none() {
            let trace_context = Context::map_current(|cx| {
                cx.has_active_span()
                    .then(|| TraceContext::from(cx.span().span_context()))
            });

            if let Some(ref trace_context) = trace_context {
                record.trace_context = Some(trace_context.clone());
            }
        }
        if record.observed_timestamp.is_none() {
            record.observed_timestamp = Some(now());
        }

        for p in processors {
            p.emit(&mut record, &self.scope);
        }
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    fn event_enabled(&self, level: Severity, target: &str) -> bool {
        self.provider
            .log_processors()
            .iter()
            .any(|processor| processor.event_enabled(level, target, self.scope.name().as_ref()))
    }
}
