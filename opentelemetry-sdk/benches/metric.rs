use std::sync::{Arc, Weak};

use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use opentelemetry_api::{
    metrics::{Counter, MeterProvider as _, Result},
    Context, Key, KeyValue,
};
use opentelemetry_sdk::{
    metrics::{
        data::{ResourceMetrics, Temporality},
        new_view,
        reader::{AggregationSelector, MetricProducer, MetricReader, TemporalitySelector},
        Aggregation, Instrument, InstrumentKind, ManualReader, MeterProvider, Pipeline, Stream,
        View,
    },
    Resource,
};

#[derive(Clone, Debug)]
struct SharedReader(Arc<dyn MetricReader>);

impl TemporalitySelector for SharedReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

impl AggregationSelector for SharedReader {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.0.aggregation(kind)
    }
}

impl MetricReader for SharedReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.0.register_pipeline(pipeline)
    }

    fn register_producer(&self, producer: Box<dyn MetricProducer>) {
        self.0.register_producer(producer)
    }

    fn collect(&self, cx: &Context, rm: &mut ResourceMetrics) -> Result<()> {
        self.0.collect(cx, rm)
    }

    fn force_flush(&self, cx: &Context) -> Result<()> {
        self.0.force_flush(cx)
    }

    fn shutdown(&self) -> Result<()> {
        self.0.shutdown()
    }
}

fn bench_counter(view: Option<Box<dyn View>>) -> (Context, SharedReader, Counter<u64>) {
    let cx = Context::new();
    let rdr = SharedReader(Arc::new(ManualReader::builder().build()));
    let mut builder = MeterProvider::builder().with_reader(rdr.clone());
    if let Some(view) = view {
        builder = builder.with_view(view);
    }
    let provider = builder.build();
    let cntr = provider.meter("test").u64_counter("hello").init();

    (cx, rdr, cntr)
}

fn counters(c: &mut Criterion) {
    let (cx, _, cntr) = bench_counter(None);

    let mut group = c.benchmark_group("Counter");
    group.bench_function("AddNoAttrs", |b| b.iter(|| cntr.add(&cx, 1, &[])));
    group.bench_function("AddOneAttr", |b| {
        b.iter(|| cntr.add(&cx, 1, &[KeyValue::new("K", "V")]))
    });
    group.bench_function("AddInvalidAttr", |b| {
        b.iter(|| cntr.add(&cx, 1, &[KeyValue::new("", "V"), KeyValue::new("K", "V")]))
    });
    group.bench_function("AddSingleUseAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("K", v)]);
            v += 1;
        })
    });
    group.bench_function("AddSingleUseInvalid", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("", v), KeyValue::new("K", v)]);
            v += 1;
        })
    });

    let (cx, _, cntr) = bench_counter(Some(
        new_view(
            Instrument::new().name("*"),
            Stream::new().attribute_filter(|kv| kv.key == Key::new("K")),
        )
        .unwrap(),
    ));

    group.bench_function("AddSingleUseFiltered", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("L", v), KeyValue::new("K", v)]);
            v += 1;
        })
    });

    let (cx, rdr, cntr) = bench_counter(None);
    let mut rm = ResourceMetrics {
        resource: Resource::empty(),
        scope_metrics: Vec::new(),
    };

    group.bench_function("CollectOneAttr", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("K", v)]);
            let _ = rdr.collect(&cx, &mut rm);
            v += 1;
        })
    });

    group.bench_function("CollectTenAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            for i in 0..10 {
                cntr.add(&cx, 1, &[KeyValue::new("K", i)]);
            }
            let _ = rdr.collect(&cx, &mut rm);
            v += 1;
        })
    });
}

fn histograms(c: &mut Criterion) {
    let mut group = c.benchmark_group("Histogram");

    group.bench_function("CollectOne", |b| benchmark_collect_histogram(b, 1));
    group.bench_function("CollectFive", |b| benchmark_collect_histogram(b, 5));
    group.bench_function("CollectTen", |b| benchmark_collect_histogram(b, 10));
    group.bench_function("CollectTwentyFive", |b| benchmark_collect_histogram(b, 25));
}

fn benchmark_collect_histogram(b: &mut Bencher, n: usize) {
    let cx = Context::new();
    let r = SharedReader(Arc::new(ManualReader::default()));
    let mtr = MeterProvider::builder()
        .with_reader(r.clone())
        .build()
        .meter("sdk/metric/bench/histogram");

    for i in 0..n {
        let h = mtr.i64_histogram(format!("fake_data_{i}")).init();
        h.record(&cx, 1, &[]);
    }

    let mut rm = ResourceMetrics {
        resource: Resource::empty(),
        scope_metrics: Vec::new(),
    };

    b.iter(|| {
        let _ = r.collect(&cx, &mut rm);
        assert_eq!(rm.scope_metrics[0].metrics.len(), n);
    })
}

criterion_group!(benches, counters, histograms);
criterion_main!(benches);
