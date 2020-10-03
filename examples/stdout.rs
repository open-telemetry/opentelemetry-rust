use opentelemetry::{
    api::Tracer,
    exporter::trace::stdout,
    sdk::{trace, Sampler},
};

fn main() {
    // Install stdout exporter pipeline to be able to retrieve collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let (tracer, _uninstall) = stdout::new_pipeline()
        .with_trace_config(trace::config().with_default_sampler(Sampler::AlwaysOn))
        .install();

    tracer.in_span("operation", |_cx| {});
}
