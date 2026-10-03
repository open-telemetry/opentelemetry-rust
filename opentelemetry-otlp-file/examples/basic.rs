//! Writes traces, metrics and logs as OTLP JSON lines.
//!
//! Write every signal to stdout:
//!
//! ```shell
//! cargo run --example basic
//! ```
//!
//! Or append each signal to its own file in a directory:
//!
//! ```shell
//! cargo run --example basic -- /tmp/otel
//! ```
//!
//! This creates `traces.jsonl`, `metrics.jsonl` and `logs.jsonl` in `/tmp/otel`.

use std::error::Error;
use std::path::Path;

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{TraceContextExt as _, Tracer as _, TracerProvider as _};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp_file::{LogExporter, MetricExporter, SpanExporter};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let (span_exporter, metric_exporter, log_exporter) = match std::env::args().nth(1) {
        Some(dir) => {
            let dir = Path::new(&dir);
            (
                SpanExporter::builder()
                    .with_file(dir.join("traces.jsonl"))
                    .build()?,
                MetricExporter::builder()
                    .with_file(dir.join("metrics.jsonl"))
                    .build()?,
                LogExporter::builder()
                    .with_file(dir.join("logs.jsonl"))
                    .build()?,
            )
        }
        None => (
            SpanExporter::default(),
            MetricExporter::default(),
            LogExporter::default(),
        ),
    };

    let resource = Resource::builder()
        .with_service_name("otlp-file-example")
        .build();
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(metric_exporter)
        .build();
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build();
    // Export `info` and above, which leaves out the SDK's own debug logs.
    tracing_subscriber::registry()
        .with(OpenTelemetryTracingBridge::new(&logger_provider).with_filter(LevelFilter::INFO))
        .init();

    let tracer = tracer_provider.tracer("otlp-file-example");
    let orders = meter_provider
        .meter("otlp-file-example")
        .u64_counter("orders")
        .build();

    tracer.in_span("checkout", |cx| {
        cx.span()
            .set_attribute(KeyValue::new("payment.method", "card"));
        tracer.in_span("charge card", |_| {});
        orders.add(1, &[KeyValue::new("payment.method", "card")]);
        tracing::info!(name: "order.placed", order_id = 42, "order placed");
    });

    // Shutting down exports everything that is still buffered.
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;
    Ok(())
}
