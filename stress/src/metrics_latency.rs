use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
    thread::{sleep, JoinHandle},
    time::{Duration, Instant},
    u64,
};

use opentelemetry::{metrics::MeterProvider, KeyValue};
use opentelemetry_sdk::{
    metrics::{
        data::{self, ResourceMetrics},
        reader::MetricReader,
        InstrumentKind, ManualReader, MetricError, Pipeline, SdkMeterProvider, Temporality,
    },
    Resource,
};
use rand::{rngs, Rng, SeedableRng};

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
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

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<(), MetricError> {
        self.0.collect(rm)
    }

    fn force_flush(&self) -> Result<(), MetricError> {
        self.0.force_flush()
    }

    fn shutdown(&self) -> Result<(), MetricError> {
        self.0.shutdown()
    }

    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

fn main() {
    let available_threads: usize = std::thread::available_parallelism().map_or(1, |p| p.get());

    for threads_count in [available_threads / 4, available_threads * 4] {
        println!("*** updates, using {threads_count} threads ***");
        measure_update_latency(&format!("no attribs"), threads_count, 10000000, |_i, _j| []);
        measure_update_latency(&format!("1 attrib"), threads_count, 10000000, |_i, _j| {
            [KeyValue::new("some_key", 1)]
        });

        measure_update_latency(&format!("9 attribs"), threads_count, 10000000, |_i, _j| {
            // for http.server.request.duration as defined in https://opentelemetry.io/docs/specs/semconv/http/http-metrics/
            [
                KeyValue::new("http.request.method", "GET"),
                KeyValue::new("url.scheme", "not_found"),
                KeyValue::new("error.type", 404),
                KeyValue::new("http.response.status_code", 404),
                KeyValue::new("http.route", "testing/metrics/latency"),
                KeyValue::new("network.protocol.name", "http"),
                KeyValue::new("network.protocol.version", 2),
                KeyValue::new("server.address", "example.com"),
                KeyValue::new("server.port", 8080),
            ]
        });
        println!("*** inserts, using {threads_count} threads ***");
        measure_update_latency(&format!("1 attrib"), threads_count, 1500, |i, j| {
            [KeyValue::new(format!("some_key{i}"), j as i64)]
        });

        measure_update_latency(&format!("10 attribs"), threads_count, 1500, |i, j| {
            [
                KeyValue::new(format!("random{i}"), j as i64),
                KeyValue::new("http.request.method", "GET"),
                KeyValue::new("url.scheme", "not_found"),
                KeyValue::new("error.type", 404),
                KeyValue::new("http.response.status_code", 404),
                KeyValue::new("http.route", "testing/metrics/latency"),
                KeyValue::new("network.protocol.name", "http"),
                KeyValue::new("network.protocol.version", 2),
                KeyValue::new("server.address", "example.com"),
                KeyValue::new("server.port", 8080),
            ]
        });
        println!("*** mix mostly updates (~10% inserts), using {threads_count} threads ***");
        measure_update_latency(&format!("10 attribs"), threads_count, 10000, |_i, j| {
            let randomness: i64 = 20
                - CURRENT_RNG.with(|rng| {
                    let mut rng = rng.borrow_mut();
                    rng.gen_range(0..20)
                });
            [
                KeyValue::new("random", (j / 10) as i64 + randomness),
                KeyValue::new("http.request.method", "GET"),
                KeyValue::new("url.scheme", "not_found"),
                KeyValue::new("error.type", 404),
                KeyValue::new("http.response.status_code", 404),
                KeyValue::new("http.route", "testing/metrics/latency"),
                KeyValue::new("network.protocol.name", "http"),
                KeyValue::new("network.protocol.version", 2),
                KeyValue::new("server.address", "example.com"),
                KeyValue::new("server.port", 8080),
            ]
        });
    }
}

fn measure_update_latency<const N: usize>(
    msg: &str,
    threads_count: usize,
    collect_around: u64,
    attribs: fn(usize, u64) -> [KeyValue; N],
) {
    let reader = SharedReader::new(
        ManualReader::builder()
            .with_temporality(Temporality::Delta)
            .build(),
    );
    let provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let histogram = provider.meter("test").u64_counter("hello").build();
    let mut threads = Vec::new();
    let mut stats = Vec::new();
    stats.resize_with(threads_count, || {
        Arc::new(Mutex::new(HashMap::<u64, u64>::new()))
    });
    let total_iterations = Arc::new(AtomicU64::new(0));
    let iterate_flag = Arc::new(AtomicBool::new(true));
    // run multiple threads and measure how time it takes to update metric
    for thread_idx in 0..threads_count {
        let hist = histogram.clone();
        let stat = stats[thread_idx].clone();
        let iterate_flag = iterate_flag.clone();
        let total_iterations = total_iterations.clone();
        threads.push(std::thread::spawn(move || {
            let mut stat = stat.lock().unwrap();
            let mut iter_idx = 0;
            while iterate_flag.load(Ordering::Acquire) {
                let kv = attribs(thread_idx, iter_idx);
                let start = Instant::now();
                hist.add(1, &kv);
                let curr = stat.entry(start.elapsed().as_nanos() as u64).or_default();
                *curr += 1;
                iter_idx += 1;
            }
            total_iterations.fetch_add(iter_idx, Ordering::AcqRel);
        }));
    }

    let total_measurements = collect_measurements(reader, threads, iterate_flag, collect_around);

    assert_eq!(total_measurements, total_iterations.load(Ordering::Acquire));

    print_stats(msg, stats, total_measurements);
}

fn collect_measurements(
    reader: SharedReader,
    threads: Vec<JoinHandle<()>>,
    iterate_flag: Arc<AtomicBool>,
    collect_around: u64,
) -> u64 {
    let start = Instant::now();
    let mut total_count = 0;
    let mut wait_for_next_collect = Duration::from_micros(500);
    while start.elapsed() < Duration::from_secs(1) {
        sleep(wait_for_next_collect);
        let collected = collect_and_return_count(&reader);
        // calculate wait interval so that the next collect cycle would be close to `collect_around`
        let ratio = collected as f64 / collect_around as f64;
        let clamped = if ratio > 2.0 {
            2.0
        } else if ratio < 0.5 {
            0.5
        } else {
            ratio
        };
        wait_for_next_collect =
            Duration::from_micros((wait_for_next_collect.as_micros() as f64 / clamped) as u64);
        total_count += collected;
    }
    iterate_flag.store(false, Ordering::Release);
    threads.into_iter().for_each(|t| {
        t.join().unwrap();
    });
    total_count += collect_and_return_count(&reader);
    total_count
}

fn print_stats(msg: &str, stats: Vec<Arc<Mutex<HashMap<u64, u64>>>>, total_measurements: u64) {
    let stats = stats
        .into_iter()
        .map(|s| Arc::into_inner(s).unwrap().into_inner().unwrap())
        .flat_map(|s| s.into_iter())
        .fold(BTreeMap::<u64, u64>::default(), |mut acc, (time, count)| {
            *acc.entry(time).or_default() += count;
            acc
        });

    let sum = stats.iter().fold(0, |mut acc, (&time, &count)| {
        acc += time * count;
        acc
    });

    println!("{msg}");
    println!("\titer  {}", format_count(total_measurements));
    println!(
        "\tp50   {}",
        format_time(get_percentile_value(total_measurements, &stats, 50))
    );
    println!(
        "\tp95   {}",
        format_time(get_percentile_value(total_measurements, &stats, 95))
    );
    println!(
        "\tp99   {}",
        format_time(get_percentile_value(total_measurements, &stats, 99))
    );
    println!("\tavg   {}", format_time(sum / total_measurements as u64));
    println!("\tbest  {}", format_time(*stats.iter().next().unwrap().0));
    println!("\tworst {}", format_time(*stats.iter().last().unwrap().0));
}

fn collect_and_return_count(reader: &SharedReader) -> u64 {
    let mut rm = ResourceMetrics {
        resource: Resource::empty(),
        scope_metrics: Vec::new(),
    };
    reader.collect(&mut rm).unwrap();
    rm.scope_metrics
        .into_iter()
        .flat_map(|sm| sm.metrics.into_iter())
        .flat_map(|m| {
            m.data
                .as_any()
                .downcast_ref::<data::Sum<u64>>()
                .unwrap()
                .data_points
                .clone()
                .into_iter()
        })
        .map(|dp| dp.value)
        .sum()
}

fn get_percentile_value(
    total_measurements: u64,
    stats: &BTreeMap<u64, u64>,
    percentile: u64,
) -> u64 {
    assert!(percentile > 0 && percentile < 100);
    let break_point = ((total_measurements as f64 * percentile as f64) / 100.0) as u64;
    let mut iter = stats.iter().peekable();
    let mut sum = 0;
    while let Some(left) = iter.next() {
        sum += left.1;
        if let Some(&right) = iter.peek() {
            let next_sum = sum + right.1;
            if next_sum > break_point {
                // interpolate
                let diff = (next_sum - sum) as f32;
                let ratio = (break_point - sum) as f32 / diff;
                let time_diff = (right.0 - left.0) as f32;
                return *left.0 + (time_diff * ratio) as u64;
            }
        }
    }
    0
}

fn format_count(count: u64) -> String {
    let count = count as f64;
    let (val, symbol) = if count > 1000000.0 {
        (count / 1000000.0, "M")
    } else if count > 1000.0 {
        (count / 1000.0, "K")
    } else {
        (count, "")
    };
    if val > 100.0 {
        format!("{val:>5.1}{symbol}")
    } else if val > 10.0 {
        format!("{val:>5.2}{symbol}")
    } else {
        format!("{val:>5.3}{symbol}")
    }
}

fn format_time(nanos: u64) -> String {
    let nanos = nanos as f64;
    let (val, symbol) = if nanos > 1000000.0 {
        (nanos / 1000000.0, "ms")
    } else if nanos > 1000.0 {
        (nanos / 1000.0, "μs")
    } else {
        (nanos, "ns")
    };
    if val > 100.0 {
        format!("{val:>5.1}{symbol}")
    } else if val > 10.0 {
        format!("{val:>5.2}{symbol}")
    } else {
        format!("{val:>5.3}{symbol}")
    }
}

#[test]
fn test_format_time() {
    assert_eq!("12.00ns", format_time(12));
    assert_eq!("123.0ns", format_time(123));
    assert_eq!("1.234μs", format_time(1234));
    assert_eq!("12.35μs", format_time(12349));
    assert_eq!("123.4μs", format_time(123400));
    assert_eq!("1.235ms", format_time(1234900));
    assert_eq!("12.34ms", format_time(12340000));
}
