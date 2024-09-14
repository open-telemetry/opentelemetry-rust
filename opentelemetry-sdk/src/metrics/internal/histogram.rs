use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::HistogramDataPoint;
use crate::metrics::data::{self, Aggregation, Temporality};
use opentelemetry::KeyValue;

use super::ValueMap;
use super::{Aggregator, Number};

struct BucketsConfig {
    bounds: Vec<f64>,
    record_min_max: bool,
    record_sum: bool,
}

#[derive(Default, Debug, Clone)]
struct Buckets<T> {
    counts: Vec<u64>,
    count: u64,
    total: T,
    min: T,
    max: T,
}

impl<T> Buckets<T>
where
    T: Number,
{
    fn new(size: usize) -> Self {
        Self {
            counts: vec![0; size],
            min: T::max(),
            max: T::min(),
            ..Default::default()
        }
    }
    fn update(&mut self, config: &BucketsConfig, f_value: f64, measurement: T) {
        // This search will return an index in the range `[0, bounds.len()]`, where
        // it will return `bounds.len()` if value is greater than the last element
        // of `bounds`. This aligns with the buckets in that the length of buckets
        // is `bounds.len()+1`, with the last bucket representing:
        // `(bounds[bounds.len()-1], +∞)`.
        let idx = config.bounds.partition_point(|&x| x < f_value);
        self.counts[idx] += 1;
        self.count += 1;
        if config.record_min_max {
            if measurement < self.min {
                self.min = measurement;
            }
            if measurement > self.max {
                self.max = measurement
            }
        }
        // it's very cheap to update it, even if it is not configured to record_sum
        self.total += measurement;
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

impl<T> Aggregator<T> for Mutex<Buckets<T>>
where
    T: Number,
{
    type Config = BucketsConfig;

    fn create(config: &BucketsConfig) -> Self {
        let size = config.bounds.len() + 1;
        Mutex::new(Buckets::new(size))
    }

    fn update(&self, config: &BucketsConfig, measurement: T) {
        let f_value = measurement.into_float();
        // Ignore NaN and infinity.
        if f_value.is_infinite() || f_value.is_nan() {
            return;
        }
        if let Ok(mut this) = self.lock() {
            this.update(config, f_value, measurement);
        }
    }
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number> {
    value_map: ValueMap<T, Mutex<Buckets<T>>>,
    start: Mutex<SystemTime>,
}

impl<T: Number> Histogram<T> {
    pub(crate) fn new(mut bounds: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        bounds.retain(|v| !v.is_nan());
        bounds.sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));
        Self {
            value_map: ValueMap::new(BucketsConfig {
                record_min_max,
                record_sum,
                bounds,
            }),
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        self.value_map.measure(measurement, attrs);
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
            if let Ok(ref mut b) = self.value_map.no_attribute_tracker.lock() {
                h.data_points.push(HistogramDataPoint {
                    attributes: vec![],
                    start_time: start,
                    time: t,
                    count: b.count,
                    bounds: self.value_map.config.bounds.clone(),
                    bucket_counts: b.counts.clone(),
                    sum: if self.value_map.config.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
                    min: if self.value_map.config.record_min_max {
                        Some(b.min)
                    } else {
                        None
                    },
                    max: if self.value_map.config.record_min_max {
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
                if let Ok(b) = tracker.lock() {
                    h.data_points.push(HistogramDataPoint {
                        attributes: attrs.clone(),
                        start_time: start,
                        time: t,
                        count: b.count,
                        bounds: self.value_map.config.bounds.clone(),
                        bucket_counts: b.counts.clone(),
                        sum: if self.value_map.config.record_sum {
                            b.total
                        } else {
                            T::default()
                        },
                        min: if self.value_map.config.record_min_max {
                            Some(b.min)
                        } else {
                            None
                        },
                        max: if self.value_map.config.record_min_max {
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
            if let Ok(b) = &self.value_map.no_attribute_tracker.lock() {
                h.data_points.push(HistogramDataPoint {
                    attributes: vec![],
                    start_time: start,
                    time: t,
                    count: b.count,
                    bounds: self.value_map.config.bounds.clone(),
                    bucket_counts: b.counts.clone(),
                    sum: if self.value_map.config.record_sum {
                        b.total
                    } else {
                        T::default()
                    },
                    min: if self.value_map.config.record_min_max {
                        Some(b.min)
                    } else {
                        None
                    },
                    max: if self.value_map.config.record_min_max {
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
                if let Ok(b) = tracker.lock() {
                    h.data_points.push(HistogramDataPoint {
                        attributes: attrs.clone(),
                        start_time: start,
                        time: t,
                        count: b.count,
                        bounds: self.value_map.config.bounds.clone(),
                        bucket_counts: b.counts.clone(),
                        sum: if self.value_map.config.record_sum {
                            b.total
                        } else {
                            T::default()
                        },
                        min: if self.value_map.config.record_min_max {
                            Some(b.min)
                        } else {
                            None
                        },
                        max: if self.value_map.config.record_min_max {
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
