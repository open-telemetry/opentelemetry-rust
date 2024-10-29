use std::{
    cell::RefCell,
    ops::DerefMut,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Barrier, Weak,
    },
    time::{Duration, Instant},
};

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Histogram, MeterProvider, MetricResult},
    KeyValue,
};
use opentelemetry_sdk::{
    metrics::{
        data::{ResourceMetrics, Temporality},
        reader::MetricReader,
        InstrumentKind, ManualReader, Pipeline, SdkMeterProvider,
    },
    Resource,
};

use rand::{
    rngs::{self, SmallRng},
    Rng, SeedableRng,
};

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliTemporality {
    Cumulative,
    Delta,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Measure metrics performance while collecting",
    long_about = "The purpose of this test is to see how collecing interferre with measurements.\n\
    Most of the test measure how fast is collecting phase, but more important is\n\
    that it doesn't \"stop-the-world\" while collection phase is running."
)]
struct Cli {
    /// Select collection phase temporality
    temporality: CliTemporality,
}

lazy_static! {
    pub static ref ATTRIBUTE_VALUES: [&'static str; 10] = [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10"
    ];
}

thread_local! {

    /// Store random number generator for each thread
    pub static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
}

fn main() {
    let cli = Cli::parse();
    let temporality = match cli.temporality {
        CliTemporality::Cumulative => Temporality::Cumulative,
        CliTemporality::Delta => Temporality::Delta,
    };
    let reader = SharedReader::new(
        ManualReader::builder()
            .with_temporality(temporality)
            .build(),
    );
    let provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    // use histogram, as it is a bit more complicated during
    let histogram = provider.meter("test").u64_histogram("hello").build();

    calculate_measurements_during_collection(histogram, reader).print_results();
}

fn calculate_measurements_during_collection(
    histogram: Histogram<u64>,
    reader: SharedReader,
) -> MeasurementResults {
    // we don't need to use every single CPU, better leave other CPU for operating system work,
    // so our running threads could be much more stable in performance.
    // just for the record, this is has HUGE effect on my machine (laptop intel i7-1355u)
    let num_threads = num_cpus::get() / 2;

    let mut res = MeasurementResults {
        total_measurements_count: 0,
        total_time_collecting: 0,
        num_iterations: 0,
    };
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        res.num_iterations += 1;
        let is_collecting = AtomicBool::new(false);
        let measurements_while_collecting = AtomicUsize::new(0);
        let time_while_collecting = AtomicUsize::new(0);
        let barrier = Barrier::new(num_threads + 1);
        std::thread::scope(|s| {
            // first create bunch of measurements,
            // so that collection phase wouldn't be "empty"
            let mut handles = Vec::new();
            for _t in 0..num_threads {
                handles.push(s.spawn(|| {
                    for _i in 0..1000 {
                        CURRENT_RNG.with(|rng| {
                            histogram.record(
                                1,
                                &random_attribute_set3(
                                    rng.borrow_mut().deref_mut(),
                                    ATTRIBUTE_VALUES.as_ref(),
                                ),
                            );
                        });
                    }
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }

            // simultaneously start collecting and creating more measurements
            for _ in 0..num_threads - 1 {
                s.spawn(|| {
                    barrier.wait();
                    let now = Instant::now();
                    let mut count = 0;
                    while is_collecting.load(Ordering::Acquire) {
                        CURRENT_RNG.with(|rng| {
                            histogram.record(
                                1,
                                &random_attribute_set3(
                                    rng.borrow_mut().deref_mut(),
                                    ATTRIBUTE_VALUES.as_ref(),
                                ),
                            );
                        });
                        count += 1;
                    }
                    measurements_while_collecting.fetch_add(count, Ordering::AcqRel);
                    time_while_collecting
                        .fetch_add(now.elapsed().as_micros() as usize, Ordering::AcqRel);
                });
            }

            let collect_handle = s.spawn(|| {
                let mut rm = ResourceMetrics {
                    resource: Resource::empty(),
                    scope_metrics: Vec::new(),
                };
                is_collecting.store(true, Ordering::Release);
                barrier.wait();
                reader.collect(&mut rm).unwrap();
                is_collecting.store(false, Ordering::Release);
            });
            barrier.wait();
            collect_handle.join().unwrap();
        });
        res.total_measurements_count += measurements_while_collecting.load(Ordering::Acquire);
        res.total_time_collecting += time_while_collecting.load(Ordering::Acquire);
    }
    res
}

struct MeasurementResults {
    total_measurements_count: usize,
    total_time_collecting: usize,
    num_iterations: usize,
}

impl MeasurementResults {
    fn print_results(&self) {
        println!(
            "{:>10.2} measurements/ms",
            self.total_measurements_count as f32 / (self.total_time_collecting as f32 / 1000.0f32)
        );
        println!(
            "{:>10.2} measurements/it",
            self.total_measurements_count as f32 / self.num_iterations as f32,
        );
        println!(
            "{:>10.2} μs/it",
            self.total_time_collecting as f32 / self.num_iterations as f32,
        );
    }
}

fn random_attribute_set3(rng: &mut SmallRng, values: &[&'static str]) -> [KeyValue; 3] {
    let len = values.len();
    unsafe {
        [
            KeyValue::new("attribute1", *values.get_unchecked(rng.gen_range(0..len))),
            KeyValue::new("attribute2", *values.get_unchecked(rng.gen_range(0..len))),
            KeyValue::new("attribute3", *values.get_unchecked(rng.gen_range(0..len))),
        ]
    }
}

// copy/paste from opentelemetry-sdk/benches/metric.rs
#[derive(Clone, Debug)]
pub struct SharedReader(Arc<dyn MetricReader>);

impl SharedReader {
    pub fn new<R>(reader: R) -> Self
    where
        R: MetricReader,
    {
        Self(Arc::new(reader))
    }
}

impl MetricReader for SharedReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.0.register_pipeline(pipeline)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> MetricResult<()> {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> MetricResult<()> {
        self.0.force_flush()
    }

    fn shutdown(&self) -> MetricResult<()> {
        self.0.shutdown()
    }

    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}
