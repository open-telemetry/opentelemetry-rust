use opentelemetry::exporter::trace::stdout;
use opentelemetry::{
    api::{Tracer, TracerProvider},
    global, sdk,
};

fn main() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = stdout::Builder::default().init();

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let provider = sdk::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    global::trace_provider()
        .get_tracer("component-main", None)
        .in_span("operation", |_cx| {});
}
