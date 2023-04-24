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

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        self.0.collect(rm)
    }

    fn force_flush(&self, cx: &Context) -> Result<()> {
        self.0.force_flush(cx)
    }

    fn shutdown(&self) -> Result<()> {
        self.0.shutdown()
    }
}

/// Configure delta temporality for all [InstrumentKind]
///
/// [Temporality::Delta] will be used for all instrument kinds if this
/// [TemporalitySelector] is used.
#[derive(Clone, Default, Debug)]
pub struct DeltaTemporalitySelector {
    pub(crate) _private: (),
}

impl DeltaTemporalitySelector {
    /// Create a new default temporality selector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TemporalitySelector for DeltaTemporalitySelector {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Delta
    }
}

// * Summary *

// rustc 1.68.0 (2c8cc3432 2023-03-06)
// cargo 1.68.0 (115f34552 2023-02-26), OS=Windows 11 Enterprise
// Intel(R) Core(TM) i7-8850H CPU @ 2.60GHz   2.59 GHz
// 12 logical and 6 physical cores

// Counter/AddNoAttrs      time:   [65.406 ns 65.535 ns 65.675 ns]
// Counter/AddNoAttrsDelta time:   [65.553 ns 65.761 ns 65.981 ns]
// Counter/AddOneAttr      time:   [341.55 ns 344.40 ns 347.58 ns]
// Counter/AddOneAttrDelta time:   [340.11 ns 342.42 ns 344.89 ns]
// Counter/AddThreeAttr    time:   [619.01 ns 624.16 ns 630.16 ns]
// Counter/AddThreeAttrDelta
//                         time:   [606.71 ns 611.45 ns 616.66 ns]
// Counter/AddFiveAttr     time:   [3.7551 µs 3.7813 µs 3.8094 µs]
// Counter/AddFiveAttrDelta
//                         time:   [3.7550 µs 3.7870 µs 3.8266 µs]
// Counter/AddTenAttr      time:   [4.7684 µs 4.7909 µs 4.8146 µs]
// Counter/AddTenAttrDelta time:   [4.7682 µs 4.8152 µs 4.8722 µs]
// Counter/AddInvalidAttr  time:   [469.31 ns 472.97 ns 476.92 ns]
// Counter/AddSingleUseAttrs
//                         time:   [749.15 ns 805.09 ns 868.03 ns]
// Counter/AddSingleUseInvalid
//                         time:   [693.75 ns 702.65 ns 713.20 ns]
// Counter/AddSingleUseFiltered
//                         time:   [677.00 ns 681.63 ns 686.88 ns]
// Counter/CollectOneAttr  time:   [659.29 ns 681.20 ns 708.04 ns]
// Counter/CollectTenAttrs time:   [3.5048 µs 3.5384 µs 3.5777 µs]
fn bench_counter(
    view: Option<Box<dyn View>>,
    temporality: &str,
) -> (Context, SharedReader, Counter<u64>) {
    let cx = Context::new();
    let rdr = if temporality == "cumulative" {
        SharedReader(Arc::new(ManualReader::builder().build()))
    } else {
        SharedReader(Arc::new(
            ManualReader::builder()
                .with_temporality_selector(DeltaTemporalitySelector::new())
                .build(),
        ))
    };
    let mut builder = MeterProvider::builder().with_reader(rdr.clone());
    if let Some(view) = view {
        builder = builder.with_view(view);
    }
    let provider = builder.build();
    let cntr = provider.meter("test").u64_counter("hello").init();

    (cx, rdr, cntr)
}

fn counters(c: &mut Criterion) {
    let (cx, _, cntr) = bench_counter(None, "cumulative");
    let (cx2, _, cntr2) = bench_counter(None, "delta");

    let mut group = c.benchmark_group("Counter");
    group.bench_function("AddNoAttrs", |b| b.iter(|| cntr.add(&cx, 1, &[])));
    group.bench_function("AddNoAttrsDelta", |b| b.iter(|| cntr2.add(&cx, 1, &[])));

    group.bench_function("AddOneAttr", |b| {
        b.iter(|| cntr.add(&cx, 1, &[KeyValue::new("K", "V")]))
    });
    group.bench_function("AddOneAttrDelta", |b| {
        b.iter(|| cntr2.add(&cx2, 1, &[KeyValue::new("K1", "V1")]))
    });
    group.bench_function("AddThreeAttr", |b| {
        b.iter(|| {
            cntr.add(
                &cx,
                1,
                &[
                    KeyValue::new("K2", "V2"),
                    KeyValue::new("K3", "V3"),
                    KeyValue::new("K4", "V4"),
                ],
            )
        })
    });
    group.bench_function("AddThreeAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                &cx2,
                1,
                &[
                    KeyValue::new("K2", "V2"),
                    KeyValue::new("K3", "V3"),
                    KeyValue::new("K4", "V4"),
                ],
            )
        })
    });
    group.bench_function("AddFiveAttr", |b| {
        b.iter(|| {
            cntr.add(
                &cx,
                1,
                &[
                    KeyValue::new("K5", "V5"),
                    KeyValue::new("K6", "V6"),
                    KeyValue::new("K7", "V7"),
                    KeyValue::new("K8", "V8"),
                    KeyValue::new("K9", "V9"),
                ],
            )
        })
    });
    group.bench_function("AddFiveAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                &cx2,
                1,
                &[
                    KeyValue::new("K5", "V5"),
                    KeyValue::new("K6", "V6"),
                    KeyValue::new("K7", "V7"),
                    KeyValue::new("K8", "V8"),
                    KeyValue::new("K9", "V9"),
                ],
            )
        })
    });
    group.bench_function("AddTenAttr", |b| {
        b.iter(|| {
            cntr.add(
                &cx,
                1,
                &[
                    KeyValue::new("K10", "V10"),
                    KeyValue::new("K11", "V11"),
                    KeyValue::new("K12", "V12"),
                    KeyValue::new("K13", "V13"),
                    KeyValue::new("K14", "V14"),
                    KeyValue::new("K15", "V15"),
                    KeyValue::new("K16", "V16"),
                    KeyValue::new("K17", "V17"),
                    KeyValue::new("K18", "V18"),
                    KeyValue::new("K19", "V19"),
                ],
            )
        })
    });
    group.bench_function("AddTenAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                &cx2,
                1,
                &[
                    KeyValue::new("K10", "V10"),
                    KeyValue::new("K11", "V11"),
                    KeyValue::new("K12", "V12"),
                    KeyValue::new("K13", "V13"),
                    KeyValue::new("K14", "V14"),
                    KeyValue::new("K15", "V15"),
                    KeyValue::new("K16", "V16"),
                    KeyValue::new("K17", "V17"),
                    KeyValue::new("K18", "V18"),
                    KeyValue::new("K19", "V19"),
                ],
            )
        })
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

    let (cx, _, cntr) = bench_counter(
        Some(
            new_view(
                Instrument::new().name("*"),
                Stream::new().attribute_filter(|kv| kv.key == Key::new("K")),
            )
            .unwrap(),
        ),
        "cumulative",
    );

    group.bench_function("AddSingleUseFiltered", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("L", v), KeyValue::new("K", v)]);
            v += 1;
        })
    });

    let (cx, rdr, cntr) = bench_counter(None, "cumulative");
    let mut rm = ResourceMetrics {
        resource: Resource::empty(),
        scope_metrics: Vec::new(),
    };

    group.bench_function("CollectOneAttr", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(&cx, 1, &[KeyValue::new("K", v)]);
            let _ = rdr.collect(&mut rm);
            v += 1;
        })
    });

    group.bench_function("CollectTenAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            for i in 0..10 {
                cntr.add(&cx, 1, &[KeyValue::new("K", i)]);
            }
            let _ = rdr.collect(&mut rm);
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
        let _ = r.collect(&mut rm);
        assert_eq!(rm.scope_metrics[0].metrics.len(), n);
    })
}

criterion_group!(benches, counters, histograms);
criterion_main!(benches);
