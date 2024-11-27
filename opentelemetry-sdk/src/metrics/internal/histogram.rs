use std::mem::replace;
use std::ops::DerefMut;
use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::HistogramDataPoint;
use crate::metrics::data::{self, Aggregation};
use crate::metrics::Temporality;
use opentelemetry::KeyValue;

use super::ValueMap;
use super::{Aggregator, Number};

impl<T> Aggregator for Mutex<Buckets<T>>
where
    T: Number,
{
    type InitConfig = usize;
    /// Value and bucket index
    type PreComputedValue = (T, usize);

    fn update(&self, (value, index): (T, usize)) {
        let mut buckets = self.lock().unwrap_or_else(|err| err.into_inner());

        buckets.total += value;
        buckets.count += 1;
        buckets.counts[index] += 1;
        if value < buckets.min {
            buckets.min = value;
        }
        if value > buckets.max {
            buckets.max = value
        }
    }

    fn create(count: &usize) -> Self {
        Mutex::new(Buckets::<T>::new(*count))
    }

    fn clone_and_reset(&self, count: &usize) -> Self {
        let mut current = self.lock().unwrap_or_else(|err| err.into_inner());
        Mutex::new(replace(current.deref_mut(), Buckets::new(*count)))
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
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number> {
    value_map: ValueMap<Mutex<Buckets<T>>>,
    bounds: Vec<f64>,
    record_min_max: bool,
    record_sum: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number> Histogram<T> {
    #[allow(unused_mut)]
    pub(crate) fn new(mut bounds: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        #[cfg(feature = "spec_unstable_metrics_views")]
        {
            // TODO: When views are used, validate this upfront
            bounds.retain(|v| !v.is_nan());
            bounds.sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));
        }

        let buckets_count = bounds.len() + 1;
        Histogram {
            value_map: ValueMap::new(buckets_count),
            bounds,
            record_min_max,
            record_sum,
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        let f = measurement.into_float();
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

        let prev_start = self
            .start
            .lock()
            .map(|mut start| replace(start.deref_mut(), t))
            .unwrap_or(t);

        self.value_map
            .collect_and_reset(&mut h.data_points, |attributes, aggr| {
                let b = aggr.into_inner().unwrap_or_else(|err| err.into_inner());
                HistogramDataPoint {
                    attributes,
                    start_time: prev_start,
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
            });

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
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

        let prev_start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());

        self.value_map
            .collect_readonly(&mut h.data_points, |attributes, aggr| {
                let b = aggr.lock().unwrap_or_else(|err| err.into_inner());
                HistogramDataPoint {
                    attributes,
                    start_time: prev_start,
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
            });

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_buckets_are_selected_correctly() {
        let hist = Histogram::<i64>::new(vec![1.0, 3.0, 6.0], false, false);
        for v in 1..11 {
            hist.measure(v, &[]);
        }
        let (count, dp) = hist.cumulative(None);
        let dp = dp.unwrap();
        let dp = dp.as_any().downcast_ref::<data::Histogram<i64>>().unwrap();
        assert_eq!(count, 1);
        assert_eq!(dp.data_points[0].count, 10);
        assert_eq!(dp.data_points[0].bucket_counts.len(), 4);
        assert_eq!(dp.data_points[0].bucket_counts[0], 1); // 1
        assert_eq!(dp.data_points[0].bucket_counts[1], 2); // 2, 3
        assert_eq!(dp.data_points[0].bucket_counts[2], 3); // 4, 5, 6
        assert_eq!(dp.data_points[0].bucket_counts[3], 4); // 7, 8, 9, 10
    }
}
