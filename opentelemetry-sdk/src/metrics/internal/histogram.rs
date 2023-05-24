use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::attributes::AttributeSet;
use crate::metrics::{
    aggregation,
    data::{self, Aggregation},
};
use opentelemetry_api::{global, metrics::MetricsError};

use super::{aggregator::STREAM_OVERFLOW_ATTRIBUTE_SET, Aggregator, Number};

#[derive(Default)]
struct Buckets<T> {
    counts: Vec<u64>,
    count: u64,
    sum: T,
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

    fn bin(&mut self, idx: usize, value: T) {
        self.counts[idx] += 1;
        self.count += 1;
        self.sum += value;
        if value < self.min {
            self.min = value;
        } else if value > self.max {
            self.max = value
        }
    }
}

/// Summarizes a set of measurements as an histValues with explicitly defined buckets.
struct HistValues<T> {
    bounds: Vec<f64>,
    values: Mutex<HashMap<AttributeSet, Buckets<T>>>,
}

impl<T: Number<T>> HistValues<T> {
    fn new(mut bounds: Vec<f64>) -> Self {
        bounds.retain(|v| !v.is_nan());
        bounds.sort_by(|a, b| a.partial_cmp(b).expect("NaNs filtered out"));

        HistValues {
            bounds,
            values: Mutex::new(Default::default()),
        }
    }
}

impl<T> Aggregator<T> for HistValues<T>
where
    T: Number<T>,
{
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
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

        match values.entry(attrs) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().bin(idx, measurement),
            Entry::Vacant(vacant_entry) => {
                if self.check_stream_cardinality(size) {
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
                    b.bin(idx, measurement);
                    vacant_entry.insert(b);
                } else {
                    values
                        .entry(STREAM_OVERFLOW_ATTRIBUTE_SET.clone())
                        .and_modify(|val| val.bin(idx, measurement))
                        .or_insert_with(|| {
                            let mut b = Buckets::new(self.bounds.len() + 1);
                            (b.min, b.max) = (measurement, measurement);
                            b.bin(idx, measurement);
                            b
                        });
                    global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                }
            }
        }
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        None // Never used
    }
}

/// Returns an Aggregator that summarizes a set of
/// measurements as an histogram. Each histogram is scoped by attributes and
/// the aggregation cycle the measurements were made in.
///
/// Each aggregation cycle is treated independently. When the returned
/// Aggregator's Aggregations method is called it will reset all histogram
/// counts to zero.
pub(crate) fn new_delta_histogram<T>(cfg: &aggregation::Aggregation) -> Arc<dyn Aggregator<T>>
where
    T: Number<T>,
{
    let (boundaries, record_min_max) = match cfg {
        aggregation::Aggregation::ExplicitBucketHistogram {
            boundaries,
            record_min_max,
        } => (boundaries.clone(), *record_min_max),
        _ => (Vec::new(), true),
    };

    Arc::new(DeltaHistogram {
        hist_values: HistValues::new(boundaries),
        record_min_max,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Summarizes a set of measurements made in a single aggregation cycle as an
/// histogram with explicitly defined buckets.
struct DeltaHistogram<T> {
    hist_values: HistValues<T>,
    record_min_max: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Aggregator<T> for DeltaHistogram<T> {
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.hist_values.aggregate(measurement, attrs)
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let mut values = match self.hist_values.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => return None,
        };
        let mut start = match self.start.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        let t = SystemTime::now();

        let data_points = values
            .drain()
            .map(|(a, b)| {
                let mut hdp = data::HistogramDataPoint {
                    attributes: a,
                    start_time: *start,
                    time: t,
                    count: b.count,
                    bounds: self.hist_values.bounds.clone(),
                    bucket_counts: b.counts,
                    sum: b.sum,
                    min: None,
                    max: None,
                    exemplars: vec![],
                };

                if self.record_min_max {
                    hdp.min = Some(b.min);
                    hdp.max = Some(b.max);
                }

                hdp
            })
            .collect::<Vec<data::HistogramDataPoint<T>>>();

        // The delta collection cycle resets.
        *start = t;
        drop(start);

        Some(Box::new(data::Histogram {
            temporality: data::Temporality::Delta,
            data_points,
        }))
    }
}

/// An [Aggregator] that summarizes a set of measurements as an histogram.
///
/// Each histogram is scoped by attributes.
///
/// Each aggregation cycle builds from the previous, the histogram counts are
/// the bucketed counts of all values aggregated since the returned Aggregator
/// was created.
pub(crate) fn new_cumulative_histogram<T>(cfg: &aggregation::Aggregation) -> Arc<dyn Aggregator<T>>
where
    T: Number<T>,
{
    let (boundaries, record_min_max) = match cfg {
        aggregation::Aggregation::ExplicitBucketHistogram {
            boundaries,
            record_min_max,
        } => (boundaries.clone(), *record_min_max),
        _ => (Vec::new(), true),
    };

    Arc::new(CumulativeHistogram {
        hist_values: HistValues::new(boundaries),
        record_min_max,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Summarizes a set of measurements made over all aggregation cycles as an
/// histogram with explicitly defined buckets.
struct CumulativeHistogram<T> {
    hist_values: HistValues<T>,

    record_min_max: bool,
    start: Mutex<SystemTime>,
}

impl<T> Aggregator<T> for CumulativeHistogram<T>
where
    T: Number<T>,
{
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.hist_values.aggregate(measurement, attrs)
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let mut values = match self.hist_values.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => return None,
        };
        let t = SystemTime::now();
        let start = self
            .start
            .lock()
            .map(|s| *s)
            .unwrap_or_else(|_| SystemTime::now());

        // TODO: This will use an unbounded amount of memory if there are unbounded
        // number of attribute sets being aggregated. Attribute sets that become
        // "stale" need to be forgotten so this will not overload the system.
        let data_points = values
            .iter_mut()
            .map(|(a, b)| {
                let mut hdp = data::HistogramDataPoint {
                    attributes: a.clone(),
                    start_time: start,
                    time: t,
                    count: b.count,
                    bounds: self.hist_values.bounds.clone(),
                    bucket_counts: b.counts.clone(),
                    sum: b.sum,
                    min: None,
                    max: None,
                    exemplars: vec![],
                };

                if self.record_min_max {
                    hdp.min = Some(b.min);
                    hdp.max = Some(b.max);
                }

                hdp
            })
            .collect::<Vec<data::HistogramDataPoint<T>>>();

        Some(Box::new(data::Histogram {
            temporality: data::Temporality::Cumulative,
            data_points,
        }))
    }
}
