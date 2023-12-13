use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use crate::metrics::data::{self, Aggregation, Temporality};
use crate::{attributes::AttributeSet, metrics::data::HistogramDataPoint};
use opentelemetry::{global, metrics::MetricsError};

use super::{
    aggregate::{is_under_cardinality_limit, STREAM_OVERFLOW_ATTRIBUTE_SET},
    BoundedMeasure, Number,
};

#[derive(Default)]
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
        } else if value > self.max {
            self.max = value
        }
    }
}

/// Summarizes a set of measurements with explicitly defined buckets.
struct HistValues<T> {
    record_sum: bool,
    bounds: Vec<f64>,
    values: Mutex<HashMap<AttributeSet, Buckets<T>>>,
}

impl<T: Number<T>> HistValues<T> {
    fn new(mut bounds: Vec<f64>, record_sum: bool) -> Self {
        bounds.retain(|v| !v.is_nan());
        bounds.sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));

        HistValues {
            record_sum,
            bounds,
            values: Mutex::new(Default::default()),
        }
    }
}

impl<T: Number<T>> HistValues<T> {
    fn measure(&self, measurement: T, attrs: AttributeSet) {
        let f = measurement.into_float();

        // This search will return an index in the range `[0, bounds.len()]`, where
        // it will return `bounds.len()` if value is greater than the last element
        // of `bounds`. This aligns with the buckets in that the length of buckets
        // is `bounds.len()+1`, with the last bucket representing:
        // `(bounds[bounds.len()-1], +∞)`.
        let idx = self.bounds.partition_point(|&x| x < f);

        let mut values = match self.values.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let size = values.len();

        let b = if let Some(b) = values.get_mut(&attrs) {
            b
        } else {
            // N+1 buckets. For example:
            //
            //   bounds = [0, 5, 10]
            //
            // Then,
            //
            //   buckets = (-∞, 0], (0, 5.0], (5.0, 10.0], (10.0, +∞)
            let mut b = Buckets::new(self.bounds.len() + 1);
            // Ensure min and max are recorded values (not zero), for new buckets.
            (b.min, b.max) = (measurement, measurement);

            if is_under_cardinality_limit(size) {
                values.entry(attrs).or_insert(b)
            } else {
                global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                values
                    .entry(STREAM_OVERFLOW_ATTRIBUTE_SET.clone())
                    .or_insert(b)
            }
        };

        b.bin(idx, measurement);
        if self.record_sum {
            b.sum(measurement)
        }
    }
}

/// Summarizes a set of measurements as a histogram with explicitly defined
/// buckets.
pub(crate) struct Histogram<T> {
    hist_values: HistValues<T>,
    record_min_max: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Histogram<T> {
    pub(crate) fn new(boundaries: Vec<f64>, record_min_max: bool, record_sum: bool) -> Self {
        Histogram {
            hist_values: HistValues::new(boundaries, record_sum),
            record_min_max,
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: AttributeSet) {
        self.hist_values.measure(measurement, attrs)
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let mut values = match self.hist_values.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => return (0, None),
        };
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

        let n = values.len();
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        for (a, b) in values.drain() {
            h.data_points.push(HistogramDataPoint {
                attributes: a,
                start_time: start,
                time: t,
                count: b.count,
                bounds: self.hist_values.bounds.clone(),
                bucket_counts: b.counts.clone(),
                sum: if self.hist_values.record_sum {
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

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

        (n, new_agg.map(|a| Box::new(a) as Box<_>))
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let values = match self.hist_values.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => return (0, None),
        };
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

        let n = values.len();
        if n > h.data_points.capacity() {
            h.data_points.reserve_exact(n - h.data_points.capacity());
        }

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        for (a, b) in values.iter() {
            h.data_points.push(HistogramDataPoint {
                attributes: a.clone(),
                start_time: start,
                time: t,
                count: b.count,
                bounds: self.hist_values.bounds.clone(),
                bucket_counts: b.counts.clone(),
                sum: if self.hist_values.record_sum {
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

        (n, new_agg.map(|a| Box::new(a) as Box<_>))
    }
}

pub(crate) fn generate_bound_measure<T: Number<T>>(
    histogram: &Arc<Histogram<T>>,
    attrs: AttributeSet,
) -> Arc<dyn BoundedMeasure<T>> {
    let cloned_self = histogram.clone();
    Arc::new(move |measurement: T| {
        cloned_self.measure(measurement, attrs.clone());
    })
}
