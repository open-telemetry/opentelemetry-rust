use opentelemetry::{
    global::{self, shutdown_tracer_provider},
    trace::{Span, Tracer},
};
use std::thread;
use std::time::Duration;

fn bar() {
    let tracer = global::tracer("component-bar");
    let mut span = tracer.start("bar");
    thread::sleep(Duration::from_millis(6));
    span.end()
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer = opentelemetry_zipkin::new_pipeline()
        .with_service_name("trace-demo")
        .install_simple()?;

    tracer.in_span("foo", |_cx| {
        thread::sleep(Duration::from_millis(6));
        bar();
        thread::sleep(Duration::from_millis(6));
    });

    shutdown_tracer_provider();
    Ok(())
}
