use opentelemetry::exporter::trace::stdout;
use opentelemetry::{api::Tracer, sdk};

fn main() {
    // Install stdout exporter pipeline to be able to retrieve collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let (tracer, _uninstall) = stdout::new_pipeline()
        .with_trace_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .install();

    tracer.in_span("operation", |_cx| {});
}
