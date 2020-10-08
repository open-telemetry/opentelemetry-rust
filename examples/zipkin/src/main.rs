use opentelemetry::api::trace::{Span, Tracer};
use opentelemetry::global;
use std::thread;
use std::time::Duration;
use std::ops::Deref;
use http_client::h1::H1Client;

fn bar() {
    let tracer = global::tracer("component-bar");
    let span = tracer.start("bar");
    thread::sleep(Duration::from_millis(6));
    span.end()
}

lazy_static::lazy_static! {
    static ref CLIENT: H1Client = H1Client::new();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tracer, _uninstall) = opentelemetry_zipkin::new_pipeline()
        .with_service_name("trace-demo")
        .with_client(CLIENT.deref())
        .install()?;

    tracer.in_span("foo", |_cx| {
        thread::sleep(Duration::from_millis(6));
        bar();
        thread::sleep(Duration::from_millis(6));
    });

    Ok(())
}
