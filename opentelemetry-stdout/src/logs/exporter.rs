use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::fmt;
use opentelemetry::logs::LogResult;
use opentelemetry::InstrumentationLibrary;
use opentelemetry_sdk::export::logs::LogBatch;
use opentelemetry_sdk::logs::LogRecord;
use opentelemetry_sdk::Resource;
use std::sync::atomic;

/// An OpenTelemetry exporter that writes Logs to stdout on export.
pub struct LogExporter {
    resource: Resource,
    is_shutdown: atomic::AtomicBool,
    resource_emitted: bool,
}

impl Default for LogExporter {
    fn default() -> Self {
        LogExporter {
            resource: Resource::default(),
            is_shutdown: atomic::AtomicBool::new(false),
            resource_emitted: false,
        }
    }
}

impl fmt::Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LogsExporter")
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    /// Export spans to stdout
    async fn export(&mut self, batch: LogBatch<'_>) -> LogResult<()> {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            return Err("exporter is shut down".into());
        } else {
            println!("Logs");
            if self.resource_emitted {
                print_logs(batch);
            } else {
                self.resource_emitted = true;
                println!("Resource");
                if let Some(schema_url) = self.resource.schema_url() {
                    println!("\t Resource SchemaUrl: {:?}", schema_url);
                }
                self.resource.iter().for_each(|(k, v)| {
                    println!("\t ->  {}={:?}", k, v);
                });

                print_logs(batch);
            }

            Ok(())
        }
    }

    fn shutdown(&mut self) {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
    }
}

fn print_logs(batch: LogBatch<'_>) {
    for (i, log) in batch.iter().enumerate() {
        println!("Log #{}", i);
        let (record, _library) = log;
        if let Some(event_name) = record.event_name {
            println!("\t EventName: {:?}", event_name);
        }
        if let Some(target) = &record.target {
            println!("\t Target (Scope): {:?}", target);
        }
        if let Some(trace_context) = &record.trace_context {
            println!("\t TraceId: {:?}", trace_context.trace_id);
            println!("\t SpanId: {:?}", trace_context.span_id);
        }
        if let Some(timestamp) = record.timestamp {
            let datetime: DateTime<Utc> = timestamp.into();
            println!("\t Timestamp: {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));
        }
        if let Some(timestamp) = record.observed_timestamp {
            let datetime: DateTime<Utc> = timestamp.into();
            println!(
                "\t Observed Timestamp: {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        if let Some(severity) = record.severity_text {
            println!("\t SeverityText: {:?}", severity);
        }
        if let Some(severity) = record.severity_number {
            println!("\t SeverityNumber: {:?}", severity);
        }
        if let Some(body) = &record.body {
            println!("\t Body: {:?}", body);
        }

        println!("\t Attributes:");
        for (k, v) in record.attributes_iter() {
            println!("\t\t ->  {}: {:?}", k, v);
        }
    }
}
