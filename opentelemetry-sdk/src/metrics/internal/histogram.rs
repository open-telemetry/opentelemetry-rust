use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::HistogramDataPoint;
use crate::metrics::data::{self, Aggregation, Temporality};
use opentelemetry::KeyValue;

use super::attribute_set_aggregation::{Aggregator, AttributeSetAggregation};
use super::Number;

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

impl<T> Aggregator<T> for Buckets<T>
where
    T: Number,
{
    type Config = BucketsConfig;

    fn create(config: &BucketsConfig) -> Self {
        let size = config.bounds.len() + 1;
        Buckets {
            counts: vec![0; size],
            min: T::max(),
            max: T::min(),
            ..Default::default()
        }
    }

    fn update(&mut self, config: &BucketsConfig, measurement: T) {
        let f_value = measurement.into_float();
        // Ignore NaN and infinity.
        if f_value.is_infinite() || f_value.is_nan() {
            return;
        }
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
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T: Number> {
    aggregators: AttributeSetAggregation<T, Buckets<T>>,
    start: Mutex<SystemTime>,
}

impl<T: Number> Histogram<T> {
    pub(crate) fn new(mut bounds: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        bounds.retain(|v| !v.is_nan());
        bounds.sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));
        Self {
            aggregators: AttributeSetAggregation::new(BucketsConfig {
                record_min_max,
                record_sum,
                bounds,
            }),
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        self.aggregators.measure(attrs, measurement)
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

        let config = self.aggregators.config();
        self.aggregators
            .collect_and_reset(&mut h.data_points, |attributes, buckets| {
                to_data_point(start, t, config, attributes, buckets)
            });

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

        let config = self.aggregators.config();
        self.aggregators
            .collect_readonly(&mut h.data_points, |attributes, buckets| {
                to_data_point(start, t, config, attributes, buckets)
            });

        (h.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
    }
}

fn to_data_point<T>(
    start_time: SystemTime,
    time: SystemTime,
    config: &BucketsConfig,
    attributes: Vec<KeyValue>,
    buckets: Buckets<T>,
) -> HistogramDataPoint<T>
where
    T: Default,
{
    HistogramDataPoint {
        attributes,
        start_time,
        time,
        count: buckets.count,
        bounds: config.bounds.clone(),
        bucket_counts: buckets.counts,
        sum: if config.record_sum {
            buckets.total
        } else {
            T::default()
        },
        min: if config.record_min_max {
            Some(buckets.min)
        } else {
            None
        },
        max: if config.record_min_max {
            Some(buckets.max)
        } else {
            None
        },
        exemplars: vec![],
    }
}
