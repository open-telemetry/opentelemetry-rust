use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
use opentelemetry::Key;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{InMemoryLogExporter, LogBatch, LogExporter, SdkLoggerProvider};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug)]
struct NoOpExporter {
    count: Arc<Mutex<usize>>,
}

impl NoOpExporter {
    fn new() -> Self {
        NoOpExporter {
            count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_count(&self) -> Arc<Mutex<usize>> {
        Arc::clone(&self.count)
    }
}

impl LogExporter for NoOpExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let mut count = self.count.lock().unwrap();
        *count += batch.len();
        // Simulate some processing time
        // Varying this will affect the number of logs dropped
        // and can be used to test the batch log processor
        // in a various scenarios.
        std::thread::sleep(std::time::Duration::from_millis(10));
        Ok(())
    }
}

fn main() {
    let total_logs_to_emit = 400_000_000;
    let num_threads = 4;
    // Setup ability to collect internal logs
    let inmemory_exporter = InMemoryLogExporter::default();
    let logger_provider_for_internal_logs: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_simple_exporter(inmemory_exporter.clone())
        .build();
    let filter_otel = EnvFilter::new("warn");
    let otel_layer = layer::OpenTelemetryTracingBridge::new(&logger_provider_for_internal_logs)
        .with_filter(filter_otel);

    tracing_subscriber::registry().with(otel_layer).init();

    let exporter = NoOpExporter::new();
    let count = exporter.get_count();
    {
        let logger_provider: SdkLoggerProvider = SdkLoggerProvider::builder()
            .with_batch_exporter(exporter)
            .build();
        let mut handles = vec![];

        let logs_per_thread = total_logs_to_emit / num_threads;

        for _ in 0..num_threads {
            let logger = logger_provider.logger("test_logger");
            let handle = std::thread::spawn(move || {
                for _ in 0..logs_per_thread {
                    let mut log_record = logger.create_log_record();
                    log_record.set_body("test log".into());
                    logger.emit(log_record);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        logger_provider.shutdown().unwrap();
    }

    let internal_logs = inmemory_exporter
        .get_emitted_logs()
        .expect("Logs are expected to be exported.");

    let dropped_logs_record = internal_logs
        .iter()
        .find(|log| log.record.event_name() == Some("BatchLogProcessor.LogsDropped"))
        .expect("Logs are expected to be exported.");

    let dropped_logs_count = dropped_logs_record
        .record
        .attributes_iter()
        .find(|attr| attr.0 == Key::new("dropped_logs_count"))
        .expect("Logs are expected to be exported.");
    let logs_dropped_by_sdk =
        if let opentelemetry::logs::AnyValue::String(value) = &dropped_logs_count.1 {
            // parse the string to an integer
            value.as_str().parse::<u64>().unwrap()
        } else {
            0
        };

    let count_logs_received_by_exporter = *count.lock().unwrap() as u64;

    println!("Total logs emitted: {}", total_logs_to_emit);
    println!(
        "Logs received by exporter: {}",
        count_logs_received_by_exporter
    );
    println!("Dropped logs count: {}", logs_dropped_by_sdk);

    if logs_dropped_by_sdk + count_logs_received_by_exporter == total_logs_to_emit {
        println!("Success! All logs are accounted for!");
    } else {
        println!(
            "Fail! {} logs are unaccounted for!",
            total_logs_to_emit - (logs_dropped_by_sdk + count_logs_received_by_exporter)
        );
    }
}
