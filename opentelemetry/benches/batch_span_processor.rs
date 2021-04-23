use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::runtime::Tokio;
use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::sdk::trace::{BatchSpanProcessor, EvictedHashMap, EvictedQueue, SpanProcessor};
use opentelemetry::trace::{
    NoopSpanExporter, SpanContext, SpanId, SpanKind, StatusCode, TraceId, TraceState,
};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::runtime::Runtime;

fn get_span_data() -> Vec<SpanData> {
    (0..200)
        .into_iter()
        .map(|_| SpanData {
            span_context: SpanContext::new(
                TraceId::from_u128(12),
                SpanId::from_u64(12),
                0,
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from_u64(12),
            span_kind: SpanKind::Client,
            name: Default::default(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: EvictedHashMap::new(12, 12),
            events: EvictedQueue::new(12),
            links: EvictedQueue::new(12),
            status_code: StatusCode::Unset,
            status_message: Default::default(),
            resource: None,
            instrumentation_lib: Default::default(),
        })
        .collect::<Vec<SpanData>>()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("BatchSpanProcessor");
    group.sample_size(5000);

    group.bench_function("batch span processor", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let span_processor =
                    BatchSpanProcessor::builder(NoopSpanExporter::new(), Tokio).build();
                let shared_span_processor = Arc::new(span_processor);
                let mut handles = Vec::with_capacity(10);
                for _ in 0..10 {
                    let span_processor = shared_span_processor.clone();
                    let spans = get_span_data();
                    handles.push(tokio::spawn(async move {
                        for span in spans {
                            span_processor.on_end(span);
                            tokio::task::yield_now().await;
                        }
                    }));
                }
                futures::future::join_all(handles).await;
            });
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
