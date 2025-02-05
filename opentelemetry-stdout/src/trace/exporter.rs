use chrono::{DateTime, Utc};
use core::fmt;
use futures_util::future::BoxFuture;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::trace::SpanData;
use std::sync::atomic;

use opentelemetry_sdk::resource::Resource;

/// An OpenTelemetry exporter that writes Spans to stdout on export.
pub struct SpanExporter {
    resource: Resource,
    is_shutdown: atomic::AtomicBool,
    resource_emitted: bool,
}

impl fmt::Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SpanExporter")
    }
}

impl Default for SpanExporter {
    fn default() -> Self {
        SpanExporter {
            resource: Resource::builder().build(),
            is_shutdown: atomic::AtomicBool::new(false),
            resource_emitted: false,
        }
    }
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporter {
    /// Write Spans to stdout
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, OTelSdkResult> {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Box::pin(std::future::ready(Err(OTelSdkError::AlreadyShutdown)))
        } else {
            println!("Spans");
            if self.resource_emitted {
                print_spans(batch);
            } else {
                self.resource_emitted = true;
                println!("Resource");
                if let Some(schema_url) = self.resource.schema_url() {
                    println!("\tResource SchemaUrl: {:?}", schema_url);
                }

                self.resource.iter().for_each(|(k, v)| {
                    println!("\t ->  {}={:?}", k, v);
                });

                print_spans(batch);
            }

            Box::pin(std::future::ready(Ok(())))
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

fn print_spans(batch: Vec<SpanData>) {
    for (i, span) in batch.into_iter().enumerate() {
        println!("Span #{}", i);
        println!("\tInstrumentation Scope");
        println!(
            "\t\tName         : {:?}",
            &span.instrumentation_scope.name()
        );
        if let Some(version) = &span.instrumentation_scope.version() {
            println!("\t\tVersion  : {:?}", version);
        }
        if let Some(schema_url) = &span.instrumentation_scope.schema_url() {
            println!("\t\tSchemaUrl: {:?}", schema_url);
        }
        span.instrumentation_scope
            .attributes()
            .enumerate()
            .for_each(|(index, kv)| {
                if index == 0 {
                    println!("\t\tScope Attributes:");
                }
                println!("\t\t\t ->  {}: {}", kv.key, kv.value);
            });

        println!();
        println!("\tName        : {}", &span.name);
        println!("\tTraceId     : {}", &span.span_context.trace_id());
        println!("\tSpanId      : {}", &span.span_context.span_id());
        println!("\tTraceFlags  : {:?}", &span.span_context.trace_flags());
        println!("\tParentSpanId: {}", &span.parent_span_id);
        println!("\tKind        : {:?}", &span.span_kind);

        let datetime: DateTime<Utc> = span.start_time.into();
        println!("\tStart time: {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));
        let datetime: DateTime<Utc> = span.end_time.into();
        println!("\tEnd time: {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));
        println!("\tStatus: {:?}", &span.status);

        let mut print_header = true;
        for kv in span.attributes.iter() {
            if print_header {
                println!("\tAttributes:");
                print_header = false;
            }
            println!("\t\t ->  {}: {:?}", kv.key, kv.value);
        }

        span.events.iter().enumerate().for_each(|(index, event)| {
            if index == 0 {
                println!("\tEvents:");
            }
            println!("\tEvent #{}", index);
            println!("\tName      : {}", event.name);
            let datetime: DateTime<Utc> = event.timestamp.into();
            println!("\tTimestamp : {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));

            event.attributes.iter().enumerate().for_each(|(index, kv)| {
                if index == 0 {
                    println!("\tAttributes:");
                }
                println!("\t\t ->  {}: {:?}", kv.key, kv.value);
            });
        });

        span.links.iter().enumerate().for_each(|(index, link)| {
            if index == 0 {
                println!("\tLinks:");
            }
            println!("\tLink #{}", index);
            println!("\tTraceId: {}", link.span_context.trace_id());
            println!("\tSpanId : {}", link.span_context.span_id());

            link.attributes.iter().enumerate().for_each(|(index, kv)| {
                if index == 0 {
                    println!("\tAttributes:");
                }
                println!("\t\t ->  {}: {:?}", kv.key, kv.value);
            });
        });
    }
}
