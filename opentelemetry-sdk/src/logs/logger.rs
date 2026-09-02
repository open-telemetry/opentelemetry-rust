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
            .with_description("The number of log records submitted to the SDK.")
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
        // Records emitted while telemetry is suppressed are the SDK's own
        // internal-operation logs (suppressed to prevent feedback loops). They
        // are not application intake, so `otel.sdk.log.created` intentionally
        // excludes them (counting them would create a phantom drop signal
        // against downstream metrics).
        if Context::is_current_telemetry_suppressed() {
            return;
        }

        // Count every record submitted to the SDK, before any processing, so
        // this metric is the top of the delivery funnel: records dropped by
        // downstream processing show up as a gap against downstream metrics.
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

#[cfg(all(test, feature = "experimental_metrics_bound_instruments"))]
mod self_obs {
    use crate::logs::SdkLoggerProvider;
    use crate::metrics::data::{AggregatedMetrics, MetricData};
    use crate::metrics::{InMemoryMetricExporter, SdkMeterProvider};
    use opentelemetry::logs::{Logger, LoggerProvider};

    fn sum_log_created(exporter: &InMemoryMetricExporter) -> u64 {
        let metrics = exporter.get_finished_metrics().unwrap();
        let mut total = 0u64;
        for rm in &metrics {
            for sm in &rm.scope_metrics {
                for metric in &sm.metrics {
                    if metric.name == "otel.sdk.log.created" {
                        if let AggregatedMetrics::U64(MetricData::Sum(sum)) = &metric.data {
                            for dp in sum.data_points() {
                                total += dp.value();
                            }
                        }
                    }
                }
            }
        }
        total
    }

    /// Verifies `otel.sdk.log.created` counts every record submitted to the SDK
    /// even when no processors are registered, proving the metric is a
    /// pre-processing intake count (the top of the delivery funnel) rather than
    /// a post-filter count.
    ///
    /// `#[ignore]`d because it calls `global::set_meter_provider()`, which
    /// mutates process-wide state; CI runs it in isolation via `test.sh`.
    #[test]
    #[ignore]
    fn log_created_counts_intake_without_processors() {
        let metric_exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter.clone())
            .build();
        opentelemetry::global::set_meter_provider(meter_provider.clone());

        // Provider with NO log processors registered.
        let logger_provider = SdkLoggerProvider::builder().build();
        let logger = logger_provider.logger("test");

        for _ in 0..10 {
            logger.emit(logger.create_log_record());
        }

        meter_provider.force_flush().unwrap();

        assert_eq!(
            sum_log_created(&metric_exporter),
            10,
            "expected 10 records counted at intake regardless of processors"
        );

        meter_provider.shutdown().unwrap();
    }
}
