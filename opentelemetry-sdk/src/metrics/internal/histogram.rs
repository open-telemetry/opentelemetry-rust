use std::mem::take;
use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::HistogramDataPoint;
use crate::metrics::data::{self, Aggregation, Temporality};
use opentelemetry::KeyValue;

use super::{collect_data_points_readonly, collect_data_points_reset, Number};
use super::{AtomicTracker, AtomicallyUpdate, Operation, ValueMap};

struct HistogramUpdate;

impl Operation for HistogramUpdate {
    fn update_tracker<T: Default, AT: AtomicTracker<T>>(tracker: &AT, value: T, index: usize) {
        tracker.update_histogram(index, value);
    }
}

struct HistogramTracker<T> {
    buckets: Mutex<Buckets<T>>,
}

impl<T: Number<T>> AtomicTracker<T> for HistogramTracker<T> {
    fn update_histogram(&self, index: usize, value: T) {
        let mut buckets = match self.buckets.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        buckets.bin(index, value);
        buckets.sum(value);
    }
}

impl<T: Number<T>> AtomicallyUpdate<T> for HistogramTracker<T> {
    type AtomicTracker = HistogramTracker<T>;

    fn new_atomic_tracker(buckets_count: Option<usize>) -> Self::AtomicTracker {
        let count = buckets_count.unwrap();
        HistogramTracker {
            buckets: Mutex::new(Buckets::<T>::new(count)),
        }
    }
}

struct Buckets<T> {
    counts: Vec<u64>,
    count: u64,
    total: T,
    min: T,
    max: T,
}

impl<T: Number<T>> Buckets<T> {
    /// returns buckets with `n` bins.
    fn new(n: usize) -> Buckets<T> {
        Buckets {
            counts: vec![0; n],
            min: T::max(),
            max: T::min(),
            count: 0,
            total: T::default(),
        }
    }

    fn sum(&mut self, value: T) {
        self.total += value;
    }

    fn bin(&mut self, idx: usize, value: T) {
        self.counts[idx] += 1;
        self.count += 1;
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value
        }
    }

    fn clone_and_reset(&mut self) -> Self {
        let n = self.counts.len();
        let res = Buckets {
            counts: take(&mut self.counts),
            count: self.count,
            total: self.total,
            min: self.min,
            max: self.max,
        };
        *self = Buckets::new(n);
        res
    }
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number<T>> {
    value_map: ValueMap<HistogramTracker<T>, T, HistogramUpdate>,
    bounds: Vec<f64>,
    record_min_max: bool,
    record_sum: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Histogram<T> {
    pub(crate) fn new(boundaries: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        let buckets_count = boundaries.len() + 1;
        let mut histogram = Histogram {
            value_map: ValueMap::new_with_buckets_count(buckets_count),
            bounds: boundaries,
            record_min_max,
            record_sum,
            start: Mutex::new(SystemTime::now()),
        };

        histogram.bounds.retain(|v| !v.is_nan());
        histogram
            .bounds
            .sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));

        histogram
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        let f = measurement.into_float();

        // This search will return an index in the range `[0, bounds.len()]`, where
        // it will return `bounds.len()` if value is greater than the last element
        // of `bounds`. This aligns with the buckets in that the length of buckets
        // is `bounds.len()+1`, with the last bucket representing:
        // `(bounds[bounds.len()-1], +∞)`.
        let index = self.bounds.partition_point(|&x| x < f);
        self.value_map.measure(measurement, attrs, index);
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
        let start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());
        let h = dest.and_then(|d| d.as_mut().downcast_mut::<data::Histogram<T>>());
        let mut new_agg = if h.is_none() {
            Some(data::Histogram {
                data_points: vec![],
                temporality: Temporality::Delta,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Delta;
        h.data_points.clear();

        let Ok(mut trackers) = self.value_map.trackers.write() else {
            return (0, None);
        };

        collect_data_points_reset(
            &self.value_map.no_attribs_tracker,
            &mut trackers,
            &mut h.data_points,
            |attributes, tracker| {
                let b = tracker
                    .buckets
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .clone_and_reset();
                HistogramDataPoint {
                    attributes,
                    start_time: start,
                    time: t,
                    count: b.count,
                    bounds: self.bounds.clone(),
                    bucket_counts: b.counts,
                    sum: if self.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
                    min: if self.record_min_max {
                        Some(b.min)
                    } else {
                        None
                    },
                    max: if self.record_min_max {
                        Some(b.max)
                    } else {
                        None
                    },
                    exemplars: vec![],
                }
            },
        );

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
        let start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());
        let h = dest.and_then(|d| d.as_mut().downcast_mut::<data::Histogram<T>>());
        let mut new_agg = if h.is_none() {
            Some(data::Histogram {
                data_points: vec![],
                temporality: Temporality::Cumulative,
            })
        } else {
            None
        };
        let h = h.unwrap_or_else(|| new_agg.as_mut().expect("present if h is none"));
        h.temporality = Temporality::Cumulative;
        h.data_points.clear();

        let Ok(trackers) = self.value_map.trackers.read() else {
            return (0, None);
        };

        collect_data_points_readonly(
            &self.value_map.no_attribs_tracker,
            &trackers,
            &mut h.data_points,
            |attributes, tracker| {
                let b = tracker
                    .buckets
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                HistogramDataPoint {
                    attributes,
                    start_time: start,
                    time: t,
                    count: b.count,
                    bounds: self.bounds.clone(),
                    bucket_counts: b.counts.clone(),
                    sum: if self.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
                    min: if self.record_min_max {
                        Some(b.min)
                    } else {
                        None
                    },
                    max: if self.record_min_max {
                        Some(b.max)
                    } else {
                        None
                    },
                    exemplars: vec![],
                }
            },
        );

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }
}
