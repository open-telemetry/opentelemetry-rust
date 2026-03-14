mod aggregate;
mod exponential_histogram;
mod histogram;
mod last_value;
mod precomputed_sum;
mod sum;

use core::fmt;
#[cfg(not(target_has_atomic = "64"))]
use portable_atomic::{AtomicI64, AtomicU64};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, AddAssign, Sub};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::{AtomicI64, AtomicU64};
use std::sync::{Arc, OnceLock, RwLock};

pub(crate) use aggregate::{AggregateBuilder, AggregateFns, ComputeAggregation, Measure};
#[cfg(feature = "experimental_metrics_bound_instruments")]
pub(crate) use aggregate::{BoundFallbackHandle, BoundMeasure};
pub(crate) use exponential_histogram::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};
#[cfg(feature = "experimental_metrics_bound_instruments")]
use opentelemetry::otel_debug;
use opentelemetry::{otel_warn, KeyValue};

use super::data::{AggregatedMetrics, MetricData};
use super::pipeline::DEFAULT_CARDINALITY_LIMIT;

// TODO Replace it with LazyLock once it is stable
pub(crate) static STREAM_OVERFLOW_ATTRIBUTES: OnceLock<Vec<KeyValue>> = OnceLock::new();

#[inline]
fn stream_overflow_attributes() -> &'static Vec<KeyValue> {
    STREAM_OVERFLOW_ATTRIBUTES.get_or_init(|| vec![KeyValue::new("otel.metric.overflow", true)])
}

pub(crate) trait Aggregator {
    /// A static configuration that is needed in order to initialize aggregator.
    /// E.g. bucket_size at creation time .
    type InitConfig;

    /// Some aggregators can do some computations before updating aggregator.
    /// This helps to reduce contention for aggregators because it makes
    /// [`Aggregator::update`] as short as possible.
    type PreComputedValue;

    /// Called everytime a new attribute-set is stored.
    fn create(init: &Self::InitConfig) -> Self;

    /// Called for each measurement.
    fn update(&self, value: Self::PreComputedValue);

    /// Return current value and reset this instance
    fn clone_and_reset(&self, init: &Self::InitConfig) -> Self;
}

/// Wraps an aggregator with status tracking for delta collection and bound instruments.
///
/// `has_been_updated` tracks whether the aggregator received measurements since the last
/// collection cycle. This enables in-place delta collection: only updated entries are exported,
/// and stale unbound entries are evicted to prevent unbounded memory growth.
///
/// `bound_count` tracks how many bound instrument handles reference this entry. Entries with
/// bound_count > 0 are never evicted from the map, even if they had no updates in a cycle
/// (they simply produce no export). This ensures bound handles always point to a live tracker.
pub(crate) struct TrackerEntry<A: Aggregator> {
    pub(crate) aggregator: A,
    pub(crate) has_been_updated: AtomicBool,
    pub(crate) bound_count: AtomicUsize,
}

impl<A: Aggregator> TrackerEntry<A> {
    fn new(config: &A::InitConfig) -> Self {
        TrackerEntry {
            aggregator: A::create(config),
            has_been_updated: AtomicBool::new(false),
            bound_count: AtomicUsize::new(0),
        }
    }
}

/// Map from attribute sets to their aggregator tracker entries.
type TrackerMap<A> = HashMap<Vec<KeyValue>, Arc<TrackerEntry<A>>>;

/// The storage for sums.
///
/// This structure is parametrized by an `Operation` that indicates how
/// updates to the underlying value trackers should be performed.
pub(crate) struct ValueMap<A>
where
    A: Aggregator,
{
    /// Trackers store the values associated with different attribute sets.
    trackers: RwLock<TrackerMap<A>>,

    /// Number of different attribute set stored in the `trackers` map.
    count: AtomicUsize,
    /// Tracker for values with no attributes attached.
    no_attribute_tracker: TrackerEntry<A>,
    /// Configuration for an Aggregator
    config: A::InitConfig,
    cardinality_limit: usize,
}

impl<A> ValueMap<A>
where
    A: Aggregator,
{
    pub(crate) fn config(&self) -> &A::InitConfig {
        &self.config
    }

    fn new(config: A::InitConfig, cardinality_limit: usize) -> Self {
        ValueMap {
            trackers: RwLock::new(HashMap::with_capacity(
                1 + min(DEFAULT_CARDINALITY_LIMIT, cardinality_limit),
            )),
            no_attribute_tracker: TrackerEntry::new(&config),
            count: AtomicUsize::new(0),
            config,
            cardinality_limit,
        }
    }

    /// Checks whether aggregator has hit cardinality limit for metric streams
    fn is_under_cardinality_limit(&self) -> bool {
        self.count.load(Ordering::SeqCst) < self.cardinality_limit
    }

    fn measure(&self, value: A::PreComputedValue, attributes: &[KeyValue]) {
        if attributes.is_empty() {
            self.no_attribute_tracker.aggregator.update(value);
            self.no_attribute_tracker
                .has_been_updated
                .store(true, Ordering::Release);
            return;
        }

        let Ok(trackers) = self.trackers.read() else {
            return;
        };

        // Try to retrieve and update the tracker with the attributes in the provided order first
        if let Some(tracker) = trackers.get(attributes) {
            tracker.aggregator.update(value);
            tracker.has_been_updated.store(true, Ordering::Relaxed);
            return;
        }

        // Try to retrieve and update the tracker with the attributes sorted.
        let sorted_attrs = sort_and_dedup(attributes);
        if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            tracker.aggregator.update(value);
            tracker.has_been_updated.store(true, Ordering::Relaxed);
            return;
        }

        // Give up the read lock before acquiring the write lock.
        drop(trackers);

        let Ok(mut trackers) = self.trackers.write() else {
            return;
        };

        // Recheck both the provided and sorted orders after acquiring the write lock
        // in case another thread has pushed an update in the meantime.
        if let Some(tracker) = trackers.get(attributes) {
            tracker.aggregator.update(value);
            tracker.has_been_updated.store(true, Ordering::Relaxed);
        } else if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            tracker.aggregator.update(value);
            tracker.has_been_updated.store(true, Ordering::Relaxed);
        } else if self.is_under_cardinality_limit() {
            let new_tracker = Arc::new(TrackerEntry::<A>::new(&self.config));
            new_tracker.aggregator.update(value);
            new_tracker.has_been_updated.store(true, Ordering::Relaxed);

            // Insert tracker with the attributes in the provided and sorted orders
            trackers.insert(attributes.to_vec(), new_tracker.clone());
            trackers.insert(sorted_attrs, new_tracker);

            self.count.fetch_add(1, Ordering::SeqCst);
        } else if let Some(overflow_value) = trackers.get(stream_overflow_attributes().as_slice()) {
            overflow_value.aggregator.update(value);
            overflow_value
                .has_been_updated
                .store(true, Ordering::Relaxed);
        } else {
            let new_tracker = TrackerEntry::<A>::new(&self.config);
            new_tracker.aggregator.update(value);
            new_tracker.has_been_updated.store(true, Ordering::Relaxed);
            trackers.insert(stream_overflow_attributes().clone(), Arc::new(new_tracker));
        }
    }

    /// Resolves attributes and returns a cached Arc<TrackerEntry> for bound instruments.
    /// The caller can then call `tracker.aggregator.update()` directly, bypassing the
    /// full lookup path on subsequent measurements.
    ///
    /// Returns `None` if the cardinality limit has been reached. The caller should
    /// fall back to the unbound `Measure::call()` path, which handles overflow
    /// correctly and enables automatic recovery when space opens up after delta
    /// collection evicts stale entries.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    fn bind(&self, attributes: &[KeyValue]) -> Option<Arc<TrackerEntry<A>>> {
        if attributes.is_empty() {
            let sorted_attrs: Vec<KeyValue> = vec![];
            return self.bind_sorted(sorted_attrs);
        }

        let sorted_attrs = sort_and_dedup(attributes);
        self.bind_sorted(sorted_attrs)
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    fn bind_sorted(&self, sorted_attrs: Vec<KeyValue>) -> Option<Arc<TrackerEntry<A>>> {
        // Fast path: read lock lookup
        if let Ok(trackers) = self.trackers.read() {
            if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
                tracker.bound_count.fetch_add(1, Ordering::Relaxed);
                return Some(Arc::clone(tracker));
            }
        }

        // Slow path: write lock, insert if missing
        let Ok(mut trackers) = self.trackers.write() else {
            // Lock poisoned — return None so caller falls back to unbound path
            return None;
        };

        // Recheck after acquiring write lock
        if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            tracker.bound_count.fetch_add(1, Ordering::Relaxed);
            return Some(Arc::clone(tracker));
        }

        if self.is_under_cardinality_limit() {
            let new_tracker = Arc::new(TrackerEntry::<A>::new(&self.config));
            new_tracker.bound_count.fetch_add(1, Ordering::Relaxed);
            trackers.insert(sorted_attrs, new_tracker.clone());
            self.count.fetch_add(1, Ordering::SeqCst);
            Some(new_tracker)
        } else {
            // Over cardinality limit — return None so the caller falls back to the
            // unbound Measure::call() path. This ensures:
            // 1. Overflow data is attributed correctly (same as unbound at overflow)
            // 2. Automatic recovery when delta collect evicts stale entries
            // 3. bound_count is not inflated on the overflow tracker
            //
            // TODO: If bind() is called during overflow, the handle remains in fallback
            // mode permanently even after delta collect frees space. Not an issue in
            // practice since bind() typically occurs at application startup before
            // cardinality fills up. Future options to revisit:
            // - Internally adjust bindings from overflow to normal during delta collect
            // - Exclude bind() from cardinality capping (users leveraging bind() know
            //   their bound instruments always report properly and never overflow)
            // See: https://github.com/open-telemetry/opentelemetry-rust/pull/3392#discussion_r2860376315
            otel_debug!(
                name: "BoundInstrument.CardinalityOverflow",
                message = "bind() called at cardinality limit, falling back to unbound path"
            );
            None
        }
    }

    /// Iterate through all attribute sets and populate `DataPoints` in readonly mode.
    /// This is used for synchronous instruments (Counter, Histogram, etc.) in Cumulative temporality mode,
    /// where attribute sets persist across collection cycles and [`ValueMap`] is not cleared.
    pub(crate) fn collect_readonly<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &A) -> Res,
    {
        prepare_data(dest, self.count.load(Ordering::SeqCst));
        if self
            .no_attribute_tracker
            .has_been_updated
            .load(Ordering::Acquire)
        {
            dest.push(map_fn(vec![], &self.no_attribute_tracker.aggregator));
        }

        let Ok(trackers) = self.trackers.read() else {
            return;
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.iter() {
            if seen.insert(Arc::as_ptr(tracker)) {
                dest.push(map_fn(attrs.clone(), &tracker.aggregator));
            }
        }
    }

    /// Iterate through all attribute sets in-place, populate `DataPoints` and reset.
    /// Only entries updated since the last collection (tracked via `has_been_updated`)
    /// are exported. Stale unbound entries are evicted to prevent unbounded memory growth.
    /// Bound entries (bound_count > 0) are never evicted — they persist until explicitly
    /// unbound, but produce no export when they have no updates.
    ///
    /// Used for synchronous instruments (Counter, Histogram, etc.) in Delta temporality mode.
    pub(crate) fn collect_and_reset<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &A) -> Res,
    {
        prepare_data(dest, self.count.load(Ordering::SeqCst));
        if self
            .no_attribute_tracker
            .has_been_updated
            .swap(false, Ordering::AcqRel)
        {
            dest.push(map_fn(vec![], &self.no_attribute_tracker.aggregator));
        }

        let overflow_attrs = stream_overflow_attributes();
        let mut stale_entries: Vec<Arc<TrackerEntry<A>>> = Vec::new();

        {
            let Ok(trackers) = self.trackers.read() else {
                return;
            };

            let mut seen = HashSet::new();
            for (attrs, tracker) in trackers.iter() {
                if seen.insert(Arc::as_ptr(tracker)) {
                    if tracker.has_been_updated.swap(false, Ordering::Relaxed) {
                        dest.push(map_fn(attrs.clone(), &tracker.aggregator));
                    } else if attrs.as_slice() != overflow_attrs.as_slice()
                        && tracker.bound_count.load(Ordering::Relaxed) == 0
                    {
                        // Stale and not bound — candidate for eviction
                        stale_entries.push(Arc::clone(tracker));
                    }
                }
            }
            // Read lock released here
        }

        if !stale_entries.is_empty() {
            if let Ok(mut trackers) = self.trackers.write() {
                // Re-check under write lock to avoid TOCTOU race: a measure() call between
                // dropping the read lock and acquiring the write lock could have updated
                // an entry we marked as stale.
                stale_entries.retain(|entry| !entry.has_been_updated.load(Ordering::Relaxed));

                if !stale_entries.is_empty() {
                    let stale_pointers: HashSet<*const TrackerEntry<A>> =
                        stale_entries.iter().map(Arc::as_ptr).collect();
                    trackers.retain(|_, tracker| !stale_pointers.contains(&Arc::as_ptr(tracker)));
                    self.count.fetch_sub(stale_entries.len(), Ordering::SeqCst);
                }
            }
        }
    }

    /// Iterate through all attribute sets, populate `DataPoints` and reset by draining the map.
    /// This is used for asynchronous instruments (Observable/PrecomputedSum) in both Delta and
    /// Cumulative temporality modes, where map clearing is needed for staleness detection.
    pub(crate) fn drain_and_reset<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, A) -> Res,
    {
        prepare_data(dest, self.count.load(Ordering::SeqCst));
        if self
            .no_attribute_tracker
            .has_been_updated
            .swap(false, Ordering::AcqRel)
        {
            dest.push(map_fn(
                vec![],
                self.no_attribute_tracker
                    .aggregator
                    .clone_and_reset(&self.config),
            ));
        }

        let old_trackers = {
            let Ok(mut trackers) = self.trackers.write() else {
                otel_warn!(name: "MeterProvider.InternalError", message = "Metric collection failed. Report this issue in OpenTelemetry repo.", details ="ValueMap trackers lock poisoned");
                return;
            };
            self.count.store(0, Ordering::SeqCst);
            std::mem::take(&mut *trackers)
            // Write lock released here
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in old_trackers {
            if seen.insert(Arc::as_ptr(&tracker)) {
                dest.push(map_fn(
                    attrs,
                    tracker.aggregator.clone_and_reset(&self.config),
                ));
            }
        }
    }
}

/// Clear and allocate exactly required amount of space for all attribute-sets
fn prepare_data<T>(data: &mut Vec<T>, list_len: usize) {
    data.clear();
    let total_len = list_len + 2; // to account for no_attributes case + overflow state
    if total_len > data.capacity() {
        data.reserve_exact(total_len - data.capacity());
    }
}

fn sort_and_dedup(attributes: &[KeyValue]) -> Vec<KeyValue> {
    // Use newly allocated vec here as incoming attributes are immutable so
    // cannot sort/de-dup in-place. TODO: This allocation can be avoided by
    // leveraging a ThreadLocal vec.
    let mut sorted = attributes.to_vec();
    sorted.sort_unstable_by(|a, b| a.key.cmp(&b.key));
    sorted.dedup_by(|a, b| a.key == b.key);
    sorted
}

/// Marks a type that can have a value added and retrieved atomically. Required since
/// different types have different backing atomic mechanisms
pub(crate) trait AtomicTracker<T>: Sync + Send + 'static {
    fn store(&self, _value: T);
    fn add(&self, _value: T);
    fn get_value(&self) -> T;
    fn get_and_reset_value(&self) -> T;
}

/// Marks a type that can have an atomic tracker generated for it
pub(crate) trait AtomicallyUpdate<T> {
    type AtomicTracker: AtomicTracker<T>;
    fn new_atomic_tracker(init: T) -> Self::AtomicTracker;
}

pub(crate) trait AggregatedMetricsAccess: Sized {
    /// This function is used in tests.
    #[allow(unused)]
    fn extract_metrics_data_ref(data: &AggregatedMetrics) -> Option<&MetricData<Self>>;
    fn extract_metrics_data_mut(data: &mut AggregatedMetrics) -> Option<&mut MetricData<Self>>;
    fn make_aggregated_metrics(data: MetricData<Self>) -> AggregatedMetrics;
}

pub(crate) trait Number:
    Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + PartialOrd
    + fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Default
    + Send
    + Sync
    + 'static
    + AtomicallyUpdate<Self>
    + AggregatedMetricsAccess
{
    fn min() -> Self;
    fn max() -> Self;

    fn into_float(self) -> f64;
}

impl Number for i64 {
    fn min() -> Self {
        i64::MIN
    }

    fn max() -> Self {
        i64::MAX
    }

    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number for u64 {
    fn min() -> Self {
        u64::MIN
    }

    fn max() -> Self {
        u64::MAX
    }

    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number for f64 {
    fn min() -> Self {
        f64::MIN
    }

    fn max() -> Self {
        f64::MAX
    }

    fn into_float(self) -> f64 {
        self
    }
}

impl AggregatedMetricsAccess for i64 {
    fn make_aggregated_metrics(data: MetricData<i64>) -> AggregatedMetrics {
        AggregatedMetrics::I64(data)
    }

    fn extract_metrics_data_ref(data: &AggregatedMetrics) -> Option<&MetricData<i64>> {
        if let AggregatedMetrics::I64(data) = data {
            Some(data)
        } else {
            None
        }
    }

    fn extract_metrics_data_mut(data: &mut AggregatedMetrics) -> Option<&mut MetricData<i64>> {
        if let AggregatedMetrics::I64(data) = data {
            Some(data)
        } else {
            None
        }
    }
}

impl AggregatedMetricsAccess for u64 {
    fn make_aggregated_metrics(data: MetricData<u64>) -> AggregatedMetrics {
        AggregatedMetrics::U64(data)
    }

    fn extract_metrics_data_ref(data: &AggregatedMetrics) -> Option<&MetricData<u64>> {
        if let AggregatedMetrics::U64(data) = data {
            Some(data)
        } else {
            None
        }
    }

    fn extract_metrics_data_mut(data: &mut AggregatedMetrics) -> Option<&mut MetricData<u64>> {
        if let AggregatedMetrics::U64(data) = data {
            Some(data)
        } else {
            None
        }
    }
}

impl AggregatedMetricsAccess for f64 {
    fn make_aggregated_metrics(data: MetricData<f64>) -> AggregatedMetrics {
        AggregatedMetrics::F64(data)
    }

    fn extract_metrics_data_ref(data: &AggregatedMetrics) -> Option<&MetricData<f64>> {
        if let AggregatedMetrics::F64(data) = data {
            Some(data)
        } else {
            None
        }
    }

    fn extract_metrics_data_mut(data: &mut AggregatedMetrics) -> Option<&mut MetricData<f64>> {
        if let AggregatedMetrics::F64(data) = data {
            Some(data)
        } else {
            None
        }
    }
}

impl AtomicTracker<u64> for AtomicU64 {
    fn store(&self, value: u64) {
        self.store(value, Ordering::Relaxed);
    }

    fn add(&self, value: u64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self) -> u64 {
        self.load(Ordering::Relaxed)
    }

    fn get_and_reset_value(&self) -> u64 {
        self.swap(0, Ordering::Relaxed)
    }
}

impl AtomicallyUpdate<u64> for u64 {
    type AtomicTracker = AtomicU64;

    fn new_atomic_tracker(init: u64) -> Self::AtomicTracker {
        AtomicU64::new(init)
    }
}

impl AtomicTracker<i64> for AtomicI64 {
    fn store(&self, value: i64) {
        self.store(value, Ordering::Relaxed);
    }

    fn add(&self, value: i64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self) -> i64 {
        self.load(Ordering::Relaxed)
    }

    fn get_and_reset_value(&self) -> i64 {
        self.swap(0, Ordering::Relaxed)
    }
}

impl AtomicallyUpdate<i64> for i64 {
    type AtomicTracker = AtomicI64;

    fn new_atomic_tracker(init: i64) -> Self::AtomicTracker {
        AtomicI64::new(init)
    }
}

pub(crate) struct F64AtomicTracker {
    inner: AtomicU64, // Floating points don't have true atomics, so we need to use the their binary representation to perform atomic operations
}

impl F64AtomicTracker {
    fn new(init: f64) -> Self {
        let value_as_u64 = init.to_bits();
        F64AtomicTracker {
            inner: AtomicU64::new(value_as_u64),
        }
    }
}

impl AtomicTracker<f64> for F64AtomicTracker {
    fn store(&self, value: f64) {
        let value_as_u64 = value.to_bits();
        self.inner.store(value_as_u64, Ordering::Relaxed);
    }

    fn add(&self, value: f64) {
        let mut current_value_as_u64 = self.inner.load(Ordering::Relaxed);

        loop {
            let current_value = f64::from_bits(current_value_as_u64);
            let new_value = current_value + value;
            let new_value_as_u64 = new_value.to_bits();
            match self.inner.compare_exchange(
                current_value_as_u64,
                new_value_as_u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                // Succeeded in updating the value
                Ok(_) => return,

                // Some other thread changed the value before this thread could update it.
                // Read the latest value again and try to swap it with the recomputed `new_value_as_u64`.
                Err(v) => current_value_as_u64 = v,
            }
        }
    }

    fn get_value(&self) -> f64 {
        let value_as_u64 = self.inner.load(Ordering::Relaxed);
        f64::from_bits(value_as_u64)
    }

    fn get_and_reset_value(&self) -> f64 {
        let zero_as_u64 = 0.0_f64.to_bits();
        let value = self.inner.swap(zero_as_u64, Ordering::Relaxed);
        f64::from_bits(value)
    }
}

impl AtomicallyUpdate<f64> for f64 {
    type AtomicTracker = F64AtomicTracker;

    fn new_atomic_tracker(init: f64) -> Self::AtomicTracker {
        F64AtomicTracker::new(init)
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::internal::last_value::Assign;

    use super::*;

    // Test helpers that return boxed trait objects to avoid method shadowing
    // from portable-atomic's inherent methods
    fn new_u64_tracker(init: u64) -> Box<dyn AtomicTracker<u64>> {
        Box::new(u64::new_atomic_tracker(init))
    }

    fn new_i64_tracker(init: i64) -> Box<dyn AtomicTracker<i64>> {
        Box::new(i64::new_atomic_tracker(init))
    }

    #[test]
    fn can_store_u64_atomic_value() {
        let atomic = new_u64_tracker(0);

        let value = atomic.get_value();
        assert_eq!(value, 0);

        atomic.store(25);
        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_add_and_get_u64_atomic_value() {
        let atomic = new_u64_tracker(0);
        atomic.add(15);
        atomic.add(10);

        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_reset_u64_atomic_value() {
        let atomic = new_u64_tracker(0);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_i64_atomic_value() {
        let atomic = new_i64_tracker(0);

        let value = atomic.get_value();
        assert_eq!(value, 0);

        atomic.store(-25);
        let value = atomic.get_value();
        assert_eq!(value, -25);

        atomic.store(25);
        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_add_and_get_i64_atomic_value() {
        let atomic = new_i64_tracker(0);
        atomic.add(15);
        atomic.add(-10);

        let value = atomic.get_value();
        assert_eq!(value, 5);
    }

    #[test]
    fn can_reset_i64_atomic_value() {
        let atomic = new_i64_tracker(0);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(0.0);
        let atomic_tracker = &atomic as &dyn AtomicTracker<f64>;

        let value = atomic.get_value();
        assert_eq!(value, 0.0);

        atomic_tracker.store(-15.5);
        let value = atomic.get_value();
        assert!(f64::abs(-15.5 - value) < 0.0001);

        atomic_tracker.store(25.7);
        let value = atomic.get_value();
        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_add_and_get_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(0.0);
        atomic.add(15.3);
        atomic.add(10.4);

        let value = atomic.get_value();

        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_reset_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(0.0);
        atomic.add(15.5);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert!(f64::abs(15.5 - value) < 0.0001, "Incorrect first value");
        assert!(f64::abs(0.0 - value2) < 0.0001, "Incorrect second value");
    }

    #[test]
    fn large_cardinality_limit() {
        // This is a regression test for panics that used to occur for large cardinality limits

        // Should not panic
        let _value_map = ValueMap::<Assign<i64>>::new((), usize::MAX);
    }
}
