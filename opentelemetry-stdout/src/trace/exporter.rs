use chrono::{DateTime, Utc};
use core::fmt;
use futures_util::future::BoxFuture;
use opentelemetry::trace::{Status, TraceError, TraceResult};
use opentelemetry_sdk::export::{self, trace::ExportResult};
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
            resource: Resource::default(),
            is_shutdown: atomic::AtomicBool::new(false),
            resource_emitted: false,
        }
    }
}

impl opentelemetry_sdk::export::trace::SpanExporter for SpanExporter {
    /// Write Spans to stdout
    fn export(&mut self, batch: Vec<export::trace::SpanData>) -> BoxFuture<'static, ExportResult> {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            return Box::pin(futures_util::future::ready(Err(TraceError::from(
                "exporter is shut down",
            ))));
        } else {
            if self.resource_emitted {
                print_spans(batch);
            } else {
                self.resource_emitted = true;
                println!("Resource");
                if self.resource.schema_url().is_some() {
                    println!("\t Resource SchemaUrl: {:?}", self.resource.schema_url());
                }

                self.resource.iter().for_each(|(k, v)| {
                    println!("\t {}={:?}", k, v);
                });

                print_spans(batch);
            }

            Box::pin(futures_util::future::ready(Ok(())))
        }
    }

    fn shutdown(&mut self) {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
    }
}

fn print_spans(batch: Vec<export::trace::SpanData>) {
    let mut i = 0;
    for span in batch {
        println!("Span #{}", i);
        i = i + 1;
        println!("\t Instrumentation Scope");
        println!("\t\t Name: {:?}", &span.instrumentation_lib.name);
        if let Some(version) = &span.instrumentation_lib.version {
            println!("\t\t Version: {:?}", version);
        }
        if let Some(schema_url) = &span.instrumentation_lib.schema_url {
            println!("\t\t SchemaUrl: {:?}", schema_url);
        }
        let mut print_header = true;
        for kv in &span.instrumentation_lib.attributes {
            if print_header {
                println!("\t\t Scope Attributes:");
                print_header = false;
            }
            println!("\t\t\t {}: {:?}", kv.key, kv.value);
        }
        println!("");
        println!("\t Name: {:?}", &span.name);
        println!("\t TraceId: {:?}", &span.span_context.trace_id());
        println!("\t SpanId: {:?}", &span.span_context.span_id());
        println!("\t ParentSpanId: {:?}", &span.parent_span_id);
        println!("\t Kind: {:?}", &span.span_kind);

        let datetime: DateTime<Utc> = span.start_time.into();
        println!(
            "\t Start time: {}",
            datetime.format("%Y-%m-%d %H:%M:%S%.6f")
        );
        let datetime: DateTime<Utc> = span.end_time.into();
        println!("\t End time: {}", datetime.format("%Y-%m-%d %H:%M:%S%.6f"));
        println!("\t Status: {:?}", &span.status);

        let mut print_header = true;
        for kv in span.attributes.iter() {
            if print_header {
                println!("\t Attributes:");
                println!(
                    "\t Dropped attributes count: {:?}",
                    span.dropped_attributes_count
                );
                print_header = false;
            }
            println!("\t\t {}: {:?}", kv.key, kv.value);
        }

        print_header = true;
        for event in span.events.iter() {
            if print_header {
                println!("\t Events:");
                print_header = false;
            }
            println!("\t\t Name: {:?}", event.name);
            let datetime: DateTime<Utc> = event.timestamp.into();
            println!(
                "\t\t Timestamp: {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
            let mut print_header_event_attributes = true;
            for kv in event.attributes.iter() {
                if print_header_event_attributes {
                    println!("\t\t Attributes:");
                    print_header_event_attributes = false;
                }
                println!("\t\t\t {}: {:?}", kv.key, kv.value);
            }
        }

        print_header = true;
        for link in span.links.iter() {
            if print_header {
                println!("\t Links:");
                print_header = false;
            }
            println!("\t\t TraceId: {:?}", link.span_context.trace_id());
            println!("\t\t SpanId: {:?}", link.span_context.span_id());
            println!("\t\t Attributes:");
            let mut print_header_link_attributes = true;
            for kv in link.attributes.iter() {
                if print_header_link_attributes {
                    println!("\t\t Attributes:");
                    print_header_link_attributes = false;
                }
                println!("\t\t\t {}: {:?}", kv.key, kv.value);
            }
        }
    }
}
