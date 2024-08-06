use rand::Rng;
use std::sync::{Arc, Weak};

use criterion::{criterion_group, criterion_main, Bencher, Criterion};
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

/*
    The benchmark results:
    criterion = "0.5.1"
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    | Test                           | Average time|
    |--------------------------------|-------------|
    | Counter_Add_Sorted             | 560 ns      |
    | Counter_Add_Unsorted           | 565 ns      |
    | Counter_Overflow               | 568 ns      |
    | ThreadLocal_Random_Generator_5 |  37 ns      |
*/

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
    group.bench_function("AddNoAttrs", |b| b.iter(|| cntr.add(1, &[])));
    group.bench_function("AddNoAttrsDelta", |b| b.iter(|| cntr2.add(1, &[])));

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

    const MAX_DATA_POINTS: i64 = 2000;
    let mut max_attributes: Vec<KeyValue> = Vec::new();

    for i in 0..MAX_DATA_POINTS - 2 {
        max_attributes.push(KeyValue::new(i.to_string(), i))
    }

    group.bench_function("AddOneTillMaxAttr", |b| {
        b.iter(|| cntr3.add(1, &max_attributes))
    });

    for i in MAX_DATA_POINTS..MAX_DATA_POINTS * 2 {
        max_attributes.push(KeyValue::new(i.to_string(), i))
    }

    group.bench_function("AddMaxAttr", |b| b.iter(|| cntr3.add(1, &max_attributes)));

    group.bench_function("AddInvalidAttr", |b| {
        b.iter(|| cntr.add(1, &[KeyValue::new("", "V"), KeyValue::new("K", "V")]))
    });
    group.bench_function("AddSingleUseAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(1, &[KeyValue::new("K", v)]);
            v += 1;
        })
    });
    group.bench_function("AddSingleUseInvalid", |b| {
        let mut v = 0;
        b.iter(|| {
            cntr.add(1, &[KeyValue::new("", v), KeyValue::new("K", v)]);
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
            cntr.add(1, &[KeyValue::new("L", v), KeyValue::new("K", v)]);
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
            cntr.add(1, &[KeyValue::new("K", v)]);
            let _ = rdr.collect(&mut rm);
            v += 1;
        })
    });

    group.bench_function("CollectTenAttrs", |b| {
        let mut v = 0;
        b.iter(|| {
            for i in 0..10 {
                cntr.add(1, &[KeyValue::new("K", i)]);
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
            let value: u64 = rng.gen_range(0..MAX_BOUND).try_into().unwrap();
            group.bench_function(
                format!("Record{}Attrs{}bounds", attr_size, bound_size),
                |b| b.iter(|| hist.record(value, &attributes)),
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
        h.record(1, &[]);
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
