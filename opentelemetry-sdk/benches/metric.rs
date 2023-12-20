use rand::Rng;
use std::sync::{Arc, Weak};

use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use opentelemetry::attributes::AttributeSet;
use opentelemetry::{
    metrics::{Counter, Histogram, MeterProvider as _, Result},
    Key, KeyValue,
};
use opentelemetry_sdk::{
    metrics::{
        data::{ResourceMetrics, Temporality},
        new_view,
        reader::{AggregationSelector, MetricReader, TemporalitySelector},
        Aggregation, Instrument, InstrumentKind, ManualReader, Pipeline, SdkMeterProvider, Stream,
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

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> Result<()> {
        self.0.force_flush()
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
// Histogram/Record0Attrs10bounds
//                         time:   [75.790 ns 77.235 ns 78.825 ns]
// Histogram/Record3Attrs10bounds
//                         time:   [580.88 ns 603.84 ns 628.71 ns]
// Histogram/Record5Attrs10bounds
//                         time:   [3.8539 µs 3.8988 µs 3.9519 µs]
// Histogram/Record7Attrs10bounds
//                         time:   [699.46 ns 720.17 ns 742.24 ns]
// Histogram/Record10Attrs10bounds
//                         time:   [844.95 ns 861.92 ns 880.23 ns]
// Histogram/Record0Attrs49bounds
//                         time:   [75.198 ns 77.081 ns 79.449 ns]
// Histogram/Record3Attrs49bounds
//                         time:   [533.82 ns 540.44 ns 547.30 ns]
// Histogram/Record5Attrs49bounds
//                         time:   [583.01 ns 588.27 ns 593.98 ns]
// Histogram/Record7Attrs49bounds
//                         time:   [645.67 ns 652.03 ns 658.35 ns]
// Histogram/Record10Attrs49bounds
//                         time:   [747.24 ns 755.42 ns 764.37 ns]
// Histogram/Record0Attrs50bounds
//                         time:   [72.023 ns 72.218 ns 72.426 ns]
// Histogram/Record3Attrs50bounds
//                         time:   [530.21 ns 534.23 ns 538.63 ns]
// Histogram/Record5Attrs50bounds
//                         time:   [3.2934 µs 3.3069 µs 3.3228 µs]
// Histogram/Record7Attrs50bounds
//                         time:   [633.88 ns 638.87 ns 644.52 ns]
// Histogram/Record10Attrs50bounds
//                         time:   [759.69 ns 768.42 ns 778.12 ns]
// Histogram/Record0Attrs1000bounds
//                         time:   [75.495 ns 75.942 ns 76.529 ns]
// Histogram/Record3Attrs1000bounds
//                         time:   [542.06 ns 548.37 ns 555.31 ns]
// Histogram/Record5Attrs1000bounds
//                         time:   [3.2935 µs 3.3058 µs 3.3215 µs]
// Histogram/Record7Attrs1000bounds
//                         time:   [643.75 ns 649.05 ns 655.14 ns]
// Histogram/Record10Attrs1000bounds
//                         time:   [726.87 ns 736.52 ns 747.09 ns]
fn bench_counter(view: Option<Box<dyn View>>, temporality: &str) -> (SharedReader, Counter<u64>) {
    let rdr = if temporality == "cumulative" {
        SharedReader(Arc::new(ManualReader::builder().build()))
    } else {
        SharedReader(Arc::new(
            ManualReader::builder()
                .with_temporality_selector(DeltaTemporalitySelector::new())
                .build(),
        ))
    };
    let mut builder = SdkMeterProvider::builder().with_reader(rdr.clone());
    if let Some(view) = view {
        builder = builder.with_view(view);
    }
    let provider = builder.build();
    let cntr = provider.meter("test").u64_counter("hello").init();

    (rdr, cntr)
}

fn counters(c: &mut Criterion) {
    let (_, cntr) = bench_counter(None, "cumulative");
    let (_, cntr2) = bench_counter(None, "delta");
    let (_, cntr3) = bench_counter(None, "cumulative");

    let mut group = c.benchmark_group("Counter");
    group.bench_function("AddNoAttrs", |b| {
        b.iter(|| cntr.add(1, AttributeSet::default()))
    });
    group.bench_function("AddNoAttrsDelta", |b| {
        b.iter(|| cntr2.add(1, AttributeSet::default()))
    });

    group.bench_function("AddOneAttr", |b| {
        b.iter(|| cntr.add(1, &[KeyValue::new("K", "V")]))
    });
    group.bench_function("AddOneAttrDelta", |b| {
        b.iter(|| cntr2.add(1, &[KeyValue::new("K1", "V1")]))
    });
    group.bench_function("AddThreeAttr", |b| {
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[
                    KeyValue::new("K2", "V2"),
                    KeyValue::new("K3", "V3"),
                    KeyValue::new("K4", "V4"),
                ]),
            )
        })
    });
    group.bench_function("AddThreeAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                1,
                AttributeSet::from(&[
                    KeyValue::new("K2", "V2"),
                    KeyValue::new("K3", "V3"),
                    KeyValue::new("K4", "V4"),
                ]),
            )
        })
    });
    group.bench_function("AddFiveAttr", |b| {
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[
                    KeyValue::new("K5", "V5"),
                    KeyValue::new("K6", "V6"),
                    KeyValue::new("K7", "V7"),
                    KeyValue::new("K8", "V8"),
                    KeyValue::new("K9", "V9"),
                ]),
            )
        })
    });
    group.bench_function("AddFiveAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                1,
                AttributeSet::from(&[
                    KeyValue::new("K5", "V5"),
                    KeyValue::new("K6", "V6"),
                    KeyValue::new("K7", "V7"),
                    KeyValue::new("K8", "V8"),
                    KeyValue::new("K9", "V9"),
                ]),
            )
        })
    });
    group.bench_function("AddTenAttr", |b| {
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[
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
                ]),
            )
        })
    });
    group.bench_function("AddTenAttrDelta", |b| {
        b.iter(|| {
            cntr2.add(
                1,
                AttributeSet::from(&[
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
                ]),
            )
        })
    });

    const MAX_DATA_POINTS: i64 = 2000;
    let mut max_attributes: Vec<KeyValue> = Vec::new();

    for i in 0..MAX_DATA_POINTS - 2 {
        max_attributes.push(KeyValue::new(i.to_string(), i))
    }

    group.bench_function("AddOneTillMaxAttr", |b| {
        b.iter(|| cntr3.add(1, max_attributes.as_slice()))
    });

    for i in MAX_DATA_POINTS..MAX_DATA_POINTS * 2 {
        max_attributes.push(KeyValue::new(i.to_string(), i))
    }

    group.bench_function("AddMaxAttr", |b| {
        b.iter(|| cntr3.add(1, max_attributes.as_slice()))
    });

    group.bench_function("AddInvalidAttr", |b| {
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[KeyValue::new("", "V"), KeyValue::new("K", "V")]),
            )
        })
    });
    group.bench_function("AddSingleUseAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(1, AttributeSet::from(&[KeyValue::new("K", v)]));
            v += 1;
        })
    });
    group.bench_function("AddSingleUseInvalid", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[KeyValue::new("", v), KeyValue::new("K", v)]),
            );
            v += 1;
        })
    });

    let (_, cntr) = bench_counter(
        Some(
            new_view(
                Instrument::new().name("*"),
                Stream::new().allowed_attribute_keys([Key::new("K")]),
            )
            .unwrap(),
        ),
        "cumulative",
    );

    group.bench_function("AddSingleUseFiltered", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(
                1,
                AttributeSet::from(&[KeyValue::new("L", v), KeyValue::new("K", v)]),
            );
            v += 1;
        })
    });

    let (rdr, cntr) = bench_counter(None, "cumulative");
    let mut rm = ResourceMetrics {
        resource: Resource::empty(),
        scope_metrics: Vec::new(),
    };

    group.bench_function("CollectOneAttr", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(1, AttributeSet::from(&[KeyValue::new("K", v)]));
            let _ = rdr.collect(&mut rm);
            v += 1;
        })
    });

    group.bench_function("CollectTenAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            for i in 0..10 {
                cntr.add(1, AttributeSet::from(&[KeyValue::new("K", i)]));
            }
            let _ = rdr.collect(&mut rm);
            v += 1;
        })
    });
}

const MAX_BOUND: usize = 100000;

fn bench_histogram(bound_count: usize) -> (SharedReader, Histogram<u64>) {
    let mut bounds = vec![0; bound_count];
    #[allow(clippy::needless_range_loop)]
    for i in 0..bounds.len() {
        bounds[i] = i * MAX_BOUND / bound_count
    }
    let view = Some(
        new_view(
            Instrument::new().name("histogram_*"),
            Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: bounds.iter().map(|&x| x as f64).collect(),
                record_min_max: true,
            }),
        )
        .unwrap(),
    );

    let r = SharedReader(Arc::new(ManualReader::default()));
    let mut builder = SdkMeterProvider::builder().with_reader(r.clone());
    if let Some(view) = view {
        builder = builder.with_view(view);
    }
    let mtr = builder.build().meter("test_meter");
    let hist = mtr
        .u64_histogram(format!("histogram_{}", bound_count))
        .init();

    (r, hist)
}

fn histograms(c: &mut Criterion) {
    let mut group = c.benchmark_group("Histogram");
    let mut rng = rand::thread_rng();

    for bound_size in [10, 49, 50, 1000].iter() {
        let (_, hist) = bench_histogram(*bound_size);
        for attr_size in [0, 3, 5, 7, 10].iter() {
            let mut attributes: Vec<KeyValue> = Vec::new();
            for i in 0..*attr_size {
                attributes.push(KeyValue::new(
                    format!("K,{},{}", bound_size, attr_size),
                    format!("V,{},{},{}", bound_size, attr_size, i),
                ))
            }

            let attributes = AttributeSet::from(&attributes);
            let value: u64 = rng.gen_range(0..MAX_BOUND).try_into().unwrap();
            group.bench_function(
                format!("Record{}Attrs{}bounds", attr_size, bound_size),
                |b| b.iter(|| hist.record(value, attributes.clone())),
            );
        }
    }
    group.bench_function("CollectOne", |b| benchmark_collect_histogram(b, 1));
    group.bench_function("CollectFive", |b| benchmark_collect_histogram(b, 5));
    group.bench_function("CollectTen", |b| benchmark_collect_histogram(b, 10));
    group.bench_function("CollectTwentyFive", |b| benchmark_collect_histogram(b, 25));
}

fn benchmark_collect_histogram(b: &mut Bencher, n: usize) {
    let r = SharedReader(Arc::new(ManualReader::default()));
    let mtr = SdkMeterProvider::builder()
        .with_reader(r.clone())
        .build()
        .meter("sdk/metric/bench/histogram");

    for i in 0..n {
        let h = mtr.u64_histogram(format!("fake_data_{i}")).init();
        h.record(1, AttributeSet::default());
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
