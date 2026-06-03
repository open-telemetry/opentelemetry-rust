#[cfg(feature = "trace")]
use super::TraceContext;
use super::{SdkLogRecord, SdkLoggerProvider};
#[cfg(feature = "trace")]
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{Context, InstrumentationScope};

use opentelemetry::logs::Severity;
use opentelemetry::time::now;

#[derive(Debug, Clone)]
/// The object for emitting [`LogRecord`]s.
///
/// [`LogRecord`]: opentelemetry::logs::LogRecord
pub struct SdkLogger {
    scope: InstrumentationScope,
    provider: SdkLoggerProvider,

    // Bound is not strictly needed (no attributes), but the semconv is still
    // `development` so the metric must be feature-gated; reuse the same
    // `experimental_metrics_bound_instruments` flag as the other SDK
    // self-observability metrics for consistency.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    log_created_counter: opentelemetry::metrics::BoundCounter<u64>,
}

impl SdkLogger {
    pub(crate) fn new(scope: InstrumentationScope, provider: SdkLoggerProvider) -> Self {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        let log_created_counter = opentelemetry::global::meter("otel.sdk")
            .u64_counter("otel.sdk.log.created")
            .with_description("The number of logs submitted to enabled SDK Loggers.")
            .with_unit("{log_record}")
            .build()
            .bind(&[]);
        SdkLogger {
            scope,
            provider,
            #[cfg(feature = "experimental_metrics_bound_instruments")]
            log_created_counter,
        }
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

        #[cfg(feature = "experimental_metrics_bound_instruments")]
        self.log_created_counter.add(1);

        let provider = &self.provider;
        let processors = provider.log_processors();

        //let mut log_record = record;
        if record.trace_context.is_none() {
            #[cfg(feature = "trace")]
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

    #[inline]
    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        if Context::is_current_telemetry_suppressed() {
            return false;
        }
        // Returns false if there are no log processors.
        // Returns true if at least one processor returns true.
        self.provider
            .log_processors()
            .iter()
            .any(|processor| processor.event_enabled(level, target, name))
    }
}
