// Measures heap allocation counts / bytes on the metrics collect path.
//
// Runs a workload of N unique attribute sets (Counter) then triggers collect()
// and reports:
//   - allocations issued during collect()
//   - bytes allocated during collect()
//
// A `CountingAllocator` wraps the system allocator globally.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{
    data::ResourceMetrics, reader::MetricReader, ManualReader, SdkMeterProvider, Temporality,
};

// ─── Counting allocator ─────────────────────────────────────────────────────

struct CountingAllocator;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static COUNTING_ON: AtomicBool = AtomicBool::new(false);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING_ON.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING_ON.load(Ordering::Relaxed) {
            DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator;

fn reset_counters() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    DEALLOC_COUNT.store(0, Ordering::Relaxed);
}

fn snapshot() -> (usize, usize, usize) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
        DEALLOC_COUNT.load(Ordering::Relaxed),
    )
}

// ─── Scenario ───────────────────────────────────────────────────────────────

fn run_scenario(name: &str, temporality: Temporality, num_streams: usize, attrs_per_stream: usize) {
    // Build meter with a manual reader so we can trigger collect directly.
    let reader: std::sync::Arc<dyn MetricReader> = std::sync::Arc::new(
        ManualReader::builder()
            .with_temporality(temporality)
            .build(),
    );
    let provider = SdkMeterProvider::builder()
        .with_reader(SharedReader(reader.clone()))
        .build();
    let counter = provider.meter("bench").u64_counter("test_counter").build();

    // Populate `num_streams` distinct attribute sets.
    for i in 0..num_streams {
        let attrs: Vec<KeyValue> = (0..attrs_per_stream)
            .map(|j| KeyValue::new(format!("k{}", j), i as i64))
            .collect();
        counter.add(1, &attrs);
    }

    // Reusable output.
    let mut rm = ResourceMetrics::default();

    // Warm one collect so any internal Vec capacity is grown.
    let _ = reader.collect(&mut rm);

    // Populate again (so the delta path has fresh data; cumulative already has it).
    for i in 0..num_streams {
        let attrs: Vec<KeyValue> = (0..attrs_per_stream)
            .map(|j| KeyValue::new(format!("k{}", j), i as i64))
            .collect();
        counter.add(1, &attrs);
    }

    // Measure allocations during collect only.
    reset_counters();
    COUNTING_ON.store(true, Ordering::Relaxed);
    let _ = reader.collect(&mut rm);
    COUNTING_ON.store(false, Ordering::Relaxed);
    let (allocs, bytes, deallocs) = snapshot();

    println!(
        "{:<40} streams={:>4} attrs={:>2} allocs={:>6} bytes={:>7} deallocs={:>6} allocs/dp={:>5.2}",
        name,
        num_streams,
        attrs_per_stream,
        allocs,
        bytes,
        deallocs,
        allocs as f64 / num_streams as f64,
    );
}

// Adapter so ManualReader can be shared behind an Arc across build().
#[derive(Clone, Debug)]
struct SharedReader(std::sync::Arc<dyn MetricReader>);

impl MetricReader for SharedReader {
    fn register_pipeline(&self, pipeline: std::sync::Weak<opentelemetry_sdk::metrics::Pipeline>) {
        self.0.register_pipeline(pipeline);
    }
    fn collect(&self, rm: &mut ResourceMetrics) -> opentelemetry_sdk::error::OTelSdkResult {
        self.0.collect(rm)
    }
    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.0.force_flush()
    }
    fn shutdown_with_timeout(
        &self,
        _timeout: std::time::Duration,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        self.0.shutdown()
    }
    fn temporality(&self, kind: opentelemetry_sdk::metrics::InstrumentKind) -> Temporality {
        self.0.temporality(kind)
    }
}

fn main() {
    println!("Metrics collect allocation profile");
    println!("==================================");

    for &(num, attrs) in &[
        (1usize, 1usize),
        (10, 1),
        (100, 1),
        (100, 3),
        (100, 5),
        (1000, 3),
        (2000, 3),
    ] {
        run_scenario("Counter/Cumulative", Temporality::Cumulative, num, attrs);
    }
    println!();
    for &(num, attrs) in &[
        (1usize, 1usize),
        (10, 1),
        (100, 1),
        (100, 3),
        (100, 5),
        (1000, 3),
        (2000, 3),
    ] {
        run_scenario("Counter/Delta     ", Temporality::Delta, num, attrs);
    }
}
