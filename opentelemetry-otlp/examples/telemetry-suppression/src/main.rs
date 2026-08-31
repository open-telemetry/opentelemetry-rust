use opentelemetry::Context;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::{
    logs::{log_processor_with_async_runtime::BatchLogProcessor, SdkLoggerProvider},
    runtime,
};
use std::{cell::RefCell, error::Error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

thread_local! {
    static SUPPRESS_GUARD: RefCell<Option<opentelemetry::ContextGuard>> =
        const { RefCell::new(None) };
}

fn main() -> Result<(), Box<dyn Error>> {
    let export_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("otel-export-runtime")
        .enable_all()
        .on_thread_start(|| {
            let guard = Context::enter_telemetry_suppressed_scope();
            SUPPRESS_GUARD.with(|slot| *slot.borrow_mut() = Some(guard));
        })
        .on_thread_stop(|| {
            SUPPRESS_GUARD.with(|slot| drop(slot.borrow_mut().take()));
        })
        .build()?;

    // Do not run application tasks on export_runtime: telemetry is suppressed
    // on all of its worker and blocking threads.
    let logger_provider = {
        // BatchLogProcessor calls tokio::spawn during build(), so the exporter
        // and processor must be built while the export runtime is entered.
        let _entered = export_runtime.enter();
        let exporter = LogExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .build()?;
        let processor = BatchLogProcessor::builder(exporter, runtime::Tokio).build();

        SdkLoggerProvider::builder()
            .with_log_processor(processor)
            .build()
    };

    // No target filter is needed for this OpenTelemetry layer because all
    // exporter work runs on threads where telemetry is suppressed.
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        target: "my-application",
        "application telemetry is exported normally"
    );

    // shutdown() blocks. When adapting this setup to #[tokio::main], call it
    // through tokio::task::spawn_blocking.
    logger_provider.shutdown()?;

    export_runtime.shutdown_background();
    Ok(())
}
