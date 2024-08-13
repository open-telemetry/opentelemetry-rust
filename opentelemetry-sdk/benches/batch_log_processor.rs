use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::LoggerProvider as _;
use opentelemetry::logs::Severity;
use opentelemetry::logs::{LogRecord, Logger};
use opentelemetry_sdk::export::logs::LogExporter;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor};
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::runtime::Tokio;
use std::sync::{Arc, Condvar, Mutex};
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

#[derive(Debug)]
struct MockLogExporter {
    export_signal: Arc<(Mutex<bool>, Condvar)>,
}

#[async_trait::async_trait]
impl LogExporter for MockLogExporter {
    async fn export<'a>(
        &mut self,
        _batch: Vec<std::borrow::Cow<'a, opentelemetry_sdk::export::logs::LogData>>,
    ) -> opentelemetry::logs::LogResult<()> {
        let (lock, cvar) = &*self.export_signal;
        let mut exported = lock.lock().unwrap();
        *exported = true;
        cvar.notify_all(); // Notify that the batch has been exported
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn set_resource(&mut self, _resource: &Resource) {}
}

fn benchmark_batch_log_processor(c: &mut Criterion) {
    // Benchmark 1: Measure time to emit and flush a log record
    let rt = Runtime::new().unwrap();
    c.bench_function("batch_log_processor_emit", |b| {
        rt.block_on(async {
            let exporter = NoopLogExporter;
            let provider = LoggerProvider::builder()
                .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                .build();
            let logger = provider.logger("test-logger");
            b.iter(|| {
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
                provider.force_flush();
            });
        });
    });

    // Benchmark 2: Measure time to emit enough logs to trigger export and wait for export completion
    c.bench_function("batch_log_processor_emit_and_wait", |b| {
        rt.block_on(async {
            let export_signal = Arc::new((Mutex::new(false), Condvar::new()));
            let exporter = MockLogExporter {
                export_signal: export_signal.clone(),
            };
            // Adjust these values to match the batch size and queue size you want to test
            let batch_size = 512;
            let max_queue_size = 2048;

            // Create the BatchConfig with the desired queue and batch size
            let batch_config = BatchConfigBuilder::default()
                .with_max_queue_size(max_queue_size)
                .with_max_export_batch_size(batch_size)
                .build();

            let log_processor = BatchLogProcessor::builder(exporter, Tokio)
                .with_batch_config(batch_config)
                .build();

            let provider = LoggerProvider::builder()
                .with_log_processor(log_processor)
                .build();

            let logger = provider.logger("test-logger");

            b.iter(|| {
                // Emit enough log records to trigger an automatic export
                //let rt = Runtime::new().unwrap();
                for _ in 0..batch_size {
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
                }

                // Wait for the export to complete
                let (lock, cvar) = &*export_signal;
                let mut exported = lock.lock().unwrap();
                while !*exported {
                    exported = cvar.wait(exported).unwrap();
                }
                *exported = false; // Reset for the next iteration
            });
        });
    });
}

criterion_group!(benches, benchmark_batch_log_processor);
criterion_main!(benches);
