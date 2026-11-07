use opentelemetry::trace::{Span as _, Tracer, TracerProvider};
use opentelemetry::{Context, InstrumentationScope, KeyValue};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{
    Sampler, SdkTracer, SdkTracerProvider, SimpleSpanProcessor, SpanData, SpanExporter,
};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Barrier;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug)]
struct VoidExporter;

impl SpanExporter for VoidExporter {
    fn export(&self, _batch: Vec<SpanData>) -> impl Future<Output = OTelSdkResult> + Send {
        std::future::ready(Ok(()))
    }
}

fn measure_op<F>(workers: usize, duration: Duration, op: &F) -> f64
where
    F: Fn() + Send + Sync,
{
    let start = Barrier::new(workers + 1);
    let stop = AtomicBool::new(false);

    let (total, elapsed) = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut spans = 0_u64;
                    start.wait();
                    while !stop.load(Ordering::Relaxed) {
                        op();
                        spans += 1;
                    }
                    spans
                })
            })
            .collect();

        start.wait();
        let started_at = Instant::now();
        std::thread::sleep(duration);
        stop.store(true, Ordering::Relaxed);
        let total = handles
            .into_iter()
            .map(|handle| handle.join().expect("worker panicked"))
            .sum::<u64>();
        (total, started_at.elapsed())
    });

    total as f64 / elapsed.as_secs_f64()
}

fn run_suite_op<F>(title: &str, max_workers: usize, duration: Duration, repeats: usize, op: F)
where
    F: Fn() + Send + Sync,
{
    println!("\n=== {} ===", title);
    let worker_counts: Vec<usize> = [1, 2, 4, 8]
        .into_iter()
        .filter(|w| *w <= max_workers)
        .collect();
    let mut results: std::collections::HashMap<usize, Vec<f64>> = worker_counts
        .iter()
        .map(|&w| (w, Vec::with_capacity(repeats)))
        .collect();

    // Interleave measurement rounds to reduce thermal and background drift:
    for _round in 0..repeats {
        for &workers in &worker_counts {
            let rate = measure_op(workers, duration, &op);
            results.get_mut(&workers).unwrap().push(rate);
        }
    }

    println!("workers\tmedian (spans/s)\tmin\tmax\tspeedup");
    let mut baseline = None;
    for &workers in &worker_counts {
        let rates = results.get_mut(&workers).unwrap();
        rates.sort_by(f64::total_cmp);
        let min = rates.first().copied().unwrap_or(0.0);
        let max = rates.last().copied().unwrap_or(0.0);
        let median = rates[rates.len() / 2];
        let base = *baseline.get_or_insert(median);
        println!(
            "{workers}\t{median:.0}\t{min:.0}\t{max:.0}\t{:.2}x",
            median / base
        );
    }
}

fn run_suite(
    title: &str,
    tracer: &SdkTracer,
    max_workers: usize,
    duration: Duration,
    repeats: usize,
) {
    run_suite_op(title, max_workers, duration, repeats, || {
        let mut span = tracer
            .span_builder("trace-scaling")
            .with_start_time(SystemTime::UNIX_EPOCH)
            .start(tracer);
        span.end_with_timestamp(SystemTime::UNIX_EPOCH);
    });
}

fn main() {
    let duration = Duration::from_millis(
        std::env::var("OTEL_TRACE_SCALING_DURATION_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_000),
    );
    let repeats = std::env::var("OTEL_TRACE_SCALING_REPEATS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);
    let max_workers = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);

    println!(
        "Running scaling benchmarks: duration={:?}, repeats={}, max_workers={}",
        duration, repeats, max_workers
    );

    // 1. Recording spans (AlwaysOn, no processor)
    let provider_rec = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .build();
    let tracer_rec = provider_rec.tracer("trace-scaling-recording");
    run_suite(
        "Recording Spans (No Processor)",
        &tracer_rec,
        max_workers,
        duration,
        repeats,
    );

    // 2. Non-recording spans (AlwaysOff, no processor)
    let provider_non_rec = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOff)
        .build();
    let tracer_non_rec = provider_non_rec.tracer("trace-scaling-non-recording");
    run_suite(
        "Non-Recording Spans (AlwaysOff)",
        &tracer_non_rec,
        max_workers,
        duration,
        repeats,
    );

    // 3. Non-recording spans via telemetry suppression
    let provider_suppressed = SdkTracerProvider::builder().build();
    let tracer_suppressed = provider_suppressed.tracer("trace-scaling-suppressed");
    let suppressed_cx = Context::new().with_telemetry_suppressed();
    run_suite_op(
        "Non-Recording Spans (Telemetry Suppressed)",
        max_workers,
        duration,
        repeats,
        || {
            let mut span = tracer_suppressed.start_with_context("trace-scaling", &suppressed_cx);
            span.end_with_timestamp(SystemTime::UNIX_EPOCH);
        },
    );

    // 4. Realistic recording spans (SimpleSpanProcessor + VoidExporter)
    let provider_realistic = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_span_processor(SimpleSpanProcessor::new(VoidExporter))
        .build();
    let tracer_realistic = provider_realistic.tracer("trace-scaling-realistic");
    run_suite(
        "Realistic Spans (SimpleSpanProcessor + VoidExporter)",
        &tracer_realistic,
        max_workers,
        duration,
        repeats,
    );

    // 5. Mixed 50% Recording / 50% Non-recording (TraceIdRatioBased(0.50))
    let provider_mixed_50 = SdkTracerProvider::builder()
        .with_sampler(Sampler::TraceIdRatioBased(0.50))
        .build();
    let tracer_mixed_50 = provider_mixed_50.tracer("trace-scaling-mixed-50");
    run_suite(
        "Mixed Spans (50% Recording / 50% Non-Recording)",
        &tracer_mixed_50,
        max_workers,
        duration,
        repeats,
    );

    // 6. Populated Scope Spans (AlwaysOff, 5 scope attributes)
    let provider_populated = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOff)
        .build();
    let scope = InstrumentationScope::builder("trace-scaling-populated")
        .with_version("1.2.3")
        .with_schema_url("https://opentelemetry.io/schemas/1.25.0")
        .with_attributes(vec![
            KeyValue::new("service.name", "scaling-service"),
            KeyValue::new("service.namespace", "benchmarks"),
            KeyValue::new("region", "us-east-1"),
            KeyValue::new("zone", "us-east-1a"),
            KeyValue::new("cluster.id", "c-1002"),
        ])
        .build();
    let tracer_populated = provider_populated.tracer_with_scope(scope);
    run_suite(
        "Populated Scope Spans (AlwaysOff, 5 Scope Attributes)",
        &tracer_populated,
        max_workers,
        duration,
        repeats,
    );
}
