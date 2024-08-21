use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::fmt;
use opentelemetry::InstrumentationLibrary;
use opentelemetry::{
    logs::{LogError, LogResult},
    ExportError,
};
use opentelemetry_sdk::logs::LogRecord;
use opentelemetry_sdk::Resource;
use std::io::{stdout, Write};

type Encoder =
    Box<dyn Fn(&mut dyn Write, crate::logs::transform::LogData) -> LogResult<()> + Send + Sync>;

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
    async fn export(&mut self, batch: Vec<(&LogRecord, &InstrumentationLibrary)>) -> LogResult<()> {
        if let Some(writer) = &mut self.writer {
            let result = (self.encoder)(writer, (batch, &self.resource).into()) as LogResult<()>;
            result.and_then(|_| writer.write_all(b"\n").map_err(|e| Error(e).into()))
        } else {
            println!("Logs");
            if self.resource_emitted {
                print_logs(batch);
            } else {
                self.resource_emitted = true;
                println!("Resource");
                if self.resource.schema_url().is_some() {
                    println!("\tResource SchemaUrl: {:?}", self.resource.schema_url());
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

fn print_logs(batch: Vec<Cow<'_, LogData>>) {
    for (i, log) in batch.into_iter().enumerate() {
        println!("Log #{}", i);
        if let Some(event_name) = &log.record.event_name {
            println!("\tEventName     : {}", event_name);
        }
        if let Some(target) = &log.record.target {
            println!("\tTarget (Scope): {}", target);
        }
        if let Some(trace_context) = &log.record.trace_context {
            println!("\tTraceId       : {}", trace_context.trace_id);
            println!("\tSpanId        : {}", trace_context.span_id);
        }
        if let Some(timestamp) = &log.record.timestamp {
            let datetime: DateTime<Utc> = (*timestamp).into();
            println!(
                "\tTimestamp     : {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        if let Some(timestamp) = &log.record.observed_timestamp {
            let datetime: DateTime<Utc> = (*timestamp).into();
            println!(
                "\tObserved Timestamp : {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        if let Some(severity) = &log.record.severity_text {
            println!("\tSeverityText  : {:?}", severity);
        }
        if let Some(severity) = &log.record.severity_number {
            println!("\tSeverityNumber: {:?}", severity);
        }
        if let Some(body) = &log.record.body {
            println!("\tBody          : {:?}", body);
        }

        println!("\tAttributes:");
        for (k, v) in log.record.attributes_iter() {
            println!("\t\t ->  {}: {:?}", k, v);
        }
    }
}
