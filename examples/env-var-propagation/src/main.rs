use opentelemetry::{
    global,
    propagation::{EnvVarExtractor, EnvVarInjector},
    trace::{Span, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_stdout::SpanExporter;
use std::{env, error::Error, io, process::Command};

fn init_tracer() -> SdkTracerProvider {
    // Parent and child both use W3C Trace Context when encoding env vars.
    global::set_text_map_propagator(TraceContextPropagator::new());

    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    global::set_tracer_provider(provider.clone());

    provider
}

fn run_parent() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let tracer = global::tracer("examples/env-var-propagation/parent");
    let span = tracer.start("parent");
    let span_context = span.span_context().clone();
    let cx = Context::current_with_span(span);

    println!(
        "parent trace_id={} span_id={}",
        span_context.trace_id(),
        span_context.span_id()
    );

    // Inject into a fresh environment map so only the child process receives
    // the propagated context variables.
    let mut child_env = EnvVarInjector::new();
    global::get_text_map_propagator(|propagator| propagator.inject_context(&cx, &mut child_env));

    // Re-exec the current binary in child mode to keep the example self-contained.
    let status = Command::new(env::current_exe()?)
        .arg("--child")
        .envs(child_env.into_inner())
        .status()?;

    cx.span().end();

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("child process failed with status {status}")).into())
    }
}

fn run_child() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Make the environment snapshot explicit at the extraction point instead of
    // hiding it inside the carrier implementation.
    let extractor = EnvVarExtractor::from_os_entries(env::vars_os());
    let parent_cx = global::get_text_map_propagator(|propagator| propagator.extract(&extractor));
    let remote_parent = parent_cx.span().span_context().clone();

    if !remote_parent.is_valid() {
        return Err(io::Error::other("missing propagated span context").into());
    }

    let tracer = global::tracer("examples/env-var-propagation/child");
    let mut span = tracer.start_with_context("child", &parent_cx);
    let span_context = span.span_context().clone();

    println!(
        "child trace_id={} parent_span_id={}",
        span_context.trace_id(),
        remote_parent.span_id()
    );

    span.end();

    Ok(())
}

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let provider = init_tracer();
    // The parent launches the same executable with `--child` to demonstrate
    // propagation across a real process boundary.
    let result = match env::args().nth(1).as_deref() {
        Some("--child") => run_child(),
        _ => run_parent(),
    };

    provider.shutdown()?;
    result
}
