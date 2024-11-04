use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::HistogramDataPoint;
use crate::metrics::data::{self, Aggregation, Temporality};
use opentelemetry::KeyValue;

use super::ValueMap;
use super::{Aggregator, Number};

struct HistogramTracker<T> {
    buckets: Mutex<Buckets<T>>,
}

impl<T> Aggregator for HistogramTracker<T>
where
    T: Number,
{
    type InitConfig = usize;
    /// Value and bucket index
    type PreComputedValue = (T, usize);

    fn update(&self, (value, index): (T, usize)) {
        let mut buckets = match self.buckets.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        buckets.bin(index, value);
        buckets.sum(value);
    }

    fn create(count: &usize) -> Self {
        HistogramTracker {
            buckets: Mutex::new(Buckets::<T>::new(*count)),
        }
    }
}

#[derive(Default)]
struct Buckets<T> {
    counts: Vec<u64>,
    count: u64,
    total: T,
    min: T,
    max: T,
}

impl<T: Number> Buckets<T> {
    /// returns buckets with `n` bins.
    fn new(n: usize) -> Buckets<T> {
        Buckets {
            counts: vec![0; n],
            min: T::max(),
            max: T::min(),
            ..Default::default()
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

    fn reset(&mut self) {
        for item in &mut self.counts {
            *item = 0;
        }
        self.count = Default::default();
        self.total = Default::default();
        self.min = T::max();
        self.max = T::min();
    }
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number> {
    value_map: ValueMap<HistogramTracker<T>>,
    bounds: Vec<f64>,
    record_min_max: bool,
    record_sum: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number> Histogram<T> {
    pub(crate) fn new(boundaries: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        // TODO fix the bug, by first removing NaN and only then getting buckets_count
        // once we know the reason for performance degradation
        let buckets_count = boundaries.len() + 1;
        let mut histogram = Histogram {
            value_map: ValueMap::new(buckets_count),
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
        // Ignore NaN and infinity.
        // Only makes sense if T is f64, maybe this could be no-op for other cases?
        // TODO: uncomment once we know the reason for performance degradation
        // if f.is_infinite() || f.is_nan() {
        //     return;
        // }
        // This search will return an index in the range `[0, bounds.len()]`, where
        // it will return `bounds.len()` if value is greater than the last element
        // of `bounds`. This aligns with the buckets in that the length of buckets
        // is `bounds.len()+1`, with the last bucket representing:
        // `(bounds[bounds.len()-1], +∞)`.
        let index = self.bounds.partition_point(|&x| x < f);

        self.value_map.measure((measurement, index), attrs);
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

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        if self
            .value_map
            .has_no_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            if let Ok(ref mut b) = self.value_map.no_attribute_tracker.buckets.lock() {
                h.data_points.push(HistogramDataPoint {
                    attributes: vec![],
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
                });

                b.reset();
            }
        }

        let mut trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.drain() {
            if seen.insert(Arc::as_ptr(&tracker)) {
                if let Ok(b) = tracker.buckets.lock() {
                    h.data_points.push(HistogramDataPoint {
                        attributes: attrs.clone(),
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
                    });
                }
            }
        }

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }
        self.value_map.count.store(0, Ordering::SeqCst);

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

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        if self
            .value_map
            .has_no_attribute_value
            .load(Ordering::Acquire)
        {
            if let Ok(b) = &self.value_map.no_attribute_tracker.buckets.lock() {
                h.data_points.push(HistogramDataPoint {
                    attributes: vec![],
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
                });
            }
        }

        let trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.iter() {
            if seen.insert(Arc::as_ptr(tracker)) {
                if let Ok(b) = tracker.buckets.lock() {
                    h.data_points.push(HistogramDataPoint {
                        attributes: attrs.clone(),
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
                    });
                }
            }
        }

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }
}

// TODO: uncomment once we know the reason for performance degradation
// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn when_f64_is_nan_or_infinity_then_ignore() {
//         struct Expected {
//             min: f64,
//             max: f64,
//             sum: f64,
//             count: u64,
//         }
//         impl Expected {
//             fn new(min: f64, max: f64, sum: f64, count: u64) -> Self {
//                 Expected {
//                     min,
//                     max,
//                     sum,
//                     count,
//                 }
//             }
//         }
//         struct TestCase {
//             values: Vec<f64>,
//             expected: Expected,
//         }

//         let test_cases = vec![
//             TestCase {
//                 values: vec![2.0, 4.0, 1.0],
//                 expected: Expected::new(1.0, 4.0, 7.0, 3),
//             },
//             TestCase {
//                 values: vec![2.0, 4.0, 1.0, f64::INFINITY],
//                 expected: Expected::new(1.0, 4.0, 7.0, 3),
//             },
//             TestCase {
//                 values: vec![2.0, 4.0, 1.0, -f64::INFINITY],
//                 expected: Expected::new(1.0, 4.0, 7.0, 3),
//             },
//             TestCase {
//                 values: vec![2.0, f64::NAN, 4.0, 1.0],
//                 expected: Expected::new(1.0, 4.0, 7.0, 3),
//             },
//             TestCase {
//                 values: vec![4.0, 4.0, 4.0, 2.0, 16.0, 1.0],
//                 expected: Expected::new(1.0, 16.0, 31.0, 6),
//             },
//         ];

//         for test in test_cases {
//             let h = Histogram::new(vec![], true, true);
//             for v in test.values {
//                 h.measure(v, &[]);
//             }
//             let res = h.value_map.no_attribute_tracker.buckets.lock().unwrap();
//             assert_eq!(test.expected.max, res.max);
//             assert_eq!(test.expected.min, res.min);
//             assert_eq!(test.expected.sum, res.total);
//             assert_eq!(test.expected.count, res.count);
//         }
//     }
// }
