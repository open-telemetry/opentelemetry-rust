use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::testing::trace::NoopSpanExporter;
use opentelemetry_sdk::trace::{BatchConfigBuilder, BatchSpanProcessor, SdkTracerProvider};
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BatchSpanProcessor");
    group.sample_size(50);

    for task_num in [1, 2, 4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("with {task_num} concurrent task")),
            task_num,
            |b, &task_num| {
                b.iter(|| {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        let span_processor = BatchSpanProcessor::builder(NoopSpanExporter::new())
                            .with_batch_config(
                                BatchConfigBuilder::default()
                                    .with_max_queue_size(10_000)
                                    .build(),
                            )
                            .build();
                        let provider = SdkTracerProvider::builder()
                            .with_span_processor(span_processor)
                            .build();
                        let tracer = provider.tracer("batch-span-processor-benchmark");
                        let mut handles = Vec::with_capacity(10);
                        for _ in 0..task_num {
                            let tracer = tracer.clone();
                            handles.push(tokio::spawn(async move {
                                for _ in 0..200 {
                                    let span = tracer.start("benchmark-span");
                                    drop(span);
                                    tokio::task::yield_now().await;
                                }
                            }));
                        }
                        futures_util::future::join_all(handles).await;
                        let _ = provider.shutdown();
                    });
                })
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
