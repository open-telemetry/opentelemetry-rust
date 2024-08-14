use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::LoggerProvider as _;
use opentelemetry::logs::Severity;
use opentelemetry::logs::{LogRecord, Logger};
use opentelemetry_sdk::export::logs::LogExporter;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor};
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::runtime::Tokio;
use std::time::Instant;
use std::time::SystemTime;
use tokio::runtime::Runtime;

#[derive(Debug)]
struct NoopLogExporter;

#[async_trait::async_trait]
impl LogExporter for NoopLogExporter {
    async fn export<'a>(
        &mut self,
        _batch: Vec<std::borrow::Cow<'a, opentelemetry_sdk::export::logs::LogData>>,
    ) -> opentelemetry::logs::LogResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn set_resource(&mut self, _resource: &Resource) {}
}

fn benchmark_batch_log_processor(c: &mut Criterion) {
    // Benchmark 1: Create record, and emit it.
    let rt = Runtime::new().unwrap();
    let max_queue_size = 10_000;
    c.bench_function("batch_log_processor_emit", |b| {
        rt.block_on(async {
            let exporter = NoopLogExporter;
            // Create the BatchConfig with the desired queue and batch size
            let batch_config = BatchConfigBuilder::default()
                .with_max_queue_size(10_000)
                .with_scheduled_delay(std::time::Duration::from_secs(1))
                .build();

            let log_processor = BatchLogProcessor::builder(exporter, Tokio)
                .with_batch_config(batch_config)
                .build();
            let provider = LoggerProvider::builder()
                .with_log_processor(log_processor)
                .build();
            let logger = provider.logger("test-logger");

            b.iter_custom(|iters| {
                let mut count = 0;
                let mut total_duration = std::time::Duration::new(0, 0);
                for _ in 0..iters {
                    if count == max_queue_size - 1 {
                        // flush the queue outside the timing loop
                        provider.force_flush();
                        count = 0;
                    }
                    let start = Instant::now();
                    let mut log_record = logger.create_log_record();
                    let now = SystemTime::now();
                    log_record.set_observed_timestamp(now);
                    log_record.set_target("my-target".to_string());
                    log_record.set_event_name("CheckoutFailed");
                    log_record.set_severity_number(Severity::Warn);
                    log_record.set_severity_text("WARN".into());
                    log_record.add_attribute("book_id", "12345");
                    log_record.add_attribute("book_title", "Rust Programming Adventures");
                    log_record.add_attribute("message", "Unable to process checkout.");
                    logger.emit(log_record);
                    count += 1;
                    total_duration += start.elapsed();
                }
                total_duration
            });
        });
    });
}

criterion_group!(benches, benchmark_batch_log_processor);
criterion_main!(benches);
