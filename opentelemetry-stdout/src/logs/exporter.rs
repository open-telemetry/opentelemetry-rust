use chrono::{DateTime, Utc};
use core::fmt;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::LogBatch;
use opentelemetry_sdk::Resource;
use std::sync::atomic;
use std::sync::atomic::Ordering;

/// An OpenTelemetry exporter that writes Logs to stdout on export.
pub struct LogExporter {
    resource: Resource,
    is_shutdown: atomic::AtomicBool,
    resource_emitted: atomic::AtomicBool,
}

impl Default for LogExporter {
    fn default() -> Self {
        LogExporter {
            resource: Resource::builder().build(),
            is_shutdown: atomic::AtomicBool::new(false),
            resource_emitted: atomic::AtomicBool::new(false),
        }
    }
}

impl fmt::Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LogExporter")
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    /// Export spans to stdout
    #[allow(clippy::manual_async_fn)]
    fn export(
        &self,
        batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        async move {
            if self.is_shutdown.load(atomic::Ordering::SeqCst) {
                Err(OTelSdkError::AlreadyShutdown)
            } else {
                println!("Logs");
                if self
                    .resource_emitted
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    print_logs(batch);
                } else {
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
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
    }
}

fn print_logs(batch: LogBatch<'_>) {
    for (i, log) in batch.iter().enumerate() {
        println!("Log #{}", i);
        let (record, library) = log;

        println!("\t Instrumentation Scope: {:?}", library);

        if let Some(event_name) = record.event_name() {
            println!("\t EventName: {:?}", event_name);
        }
        if let Some(target) = record.target() {
            println!("\t Target (Scope): {:?}", target);
        }
        if let Some(trace_context) = record.trace_context() {
            println!("\t TraceId: {:?}", trace_context.trace_id);
            println!("\t SpanId: {:?}", trace_context.span_id);
            if let Some(trace_flags) = trace_context.trace_flags {
                println!("\t TraceFlags: {:?}", trace_flags);
            }
        }
        if let Some(timestamp) = record.timestamp() {
            let datetime: DateTime<Utc> = timestamp.into();
            println!("\t Timestamp: {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));
        }
        if let Some(timestamp) = record.observed_timestamp() {
            let datetime: DateTime<Utc> = timestamp.into();
            println!(
                "\t Observed Timestamp: {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        if let Some(severity) = record.severity_text() {
            println!("\t SeverityText: {:?}", severity);
        }
        if let Some(severity) = record.severity_number() {
            println!("\t SeverityNumber: {:?}", severity);
        }
        if let Some(body) = record.body() {
            println!("\t Body: {:?}", body);
        }

        println!("\t Attributes:");
        for (k, v) in record.attributes_iter() {
            println!("\t\t ->  {}: {:?}", k, v);
        }
    }
}
