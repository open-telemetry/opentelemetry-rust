use opentelemetry::sdk::export::trace::stdout;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::{trace::Tracer, KeyValue};

fn init_tracer() -> impl Tracer {
    let v = vec![KeyValue::new("key", "value")];
    stdout::new_pipeline()
        .with_trace_config(trace::config().with_sampler(Sampler::AlwaysOn))
        .install_with_tracer_attributes(v)
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let _tracer = init_tracer();
    let _span = _tracer.start("say hello");

    Ok(())
}
