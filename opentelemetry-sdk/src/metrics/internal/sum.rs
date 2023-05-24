use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::attributes::AttributeSet;
use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};
use opentelemetry_api::{global, metrics::MetricsError};

use super::{
    aggregator::{PrecomputeAggregator, get_stream_overflow_attribute_set},
    Aggregator, Number,
};

/// The storage for sums.
#[derive(Default)]
struct ValueMap<T: Number<T>> {
    values: Mutex<HashMap<AttributeSet, T>>,
}

impl<T: Number<T>> ValueMap<T> {
    pub(crate) fn new() -> Self {
        ValueMap {
            values: Mutex::new(HashMap::new()),
        }
    }
}

impl<T: Number<T>> Aggregator<T> for ValueMap<T> {
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        if let Ok(mut values) = self.values.lock() {
            let size = values.len();
            match values.entry(attrs) {
                Entry::Occupied(mut occupied_entry) => {
                    let sum = occupied_entry.get_mut();
                    *sum += measurement;
                }
                Entry::Vacant(vacant_entry) => {
                    if self.is_under_cardinality_limit(size) {
                        vacant_entry.insert(measurement);
                    } else {
                        values
                            .entry(get_stream_overflow_attribute_set().clone())
                            .and_modify(|val| *val += measurement)
                            .or_insert(measurement);
                        global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                    }
                }
            }
        }
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        None // Never called directly
    }
}

/// Returns an [Aggregator] that summarizes a set of measurements as their
/// arithmetic sum.
///
/// Each sum is scoped by attributes and the aggregation cycle the measurements
/// were made in.
///
/// The `monotonic` value is used to specify if the produced [Aggregation] is
/// monotonic or not. The returned [Aggregator] does not make any guarantees this
/// value is accurate. It is up to the caller to ensure it.
///
/// Each aggregation cycle is treated independently. When the returned
/// [Aggregator::aggregation] method is called it will reset all sums to zero.
pub(crate) fn new_delta_sum<T>(monotonic: bool) -> Arc<dyn Aggregator<T>>
where
    T: Number<T>,
{
    Arc::new(DeltaSum {
        value_map: ValueMap::new(),
        monotonic,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Summarizes a set of measurements made in a single aggregation cycle as their
/// arithmetic sum.
struct DeltaSum<T: Number<T>> {
    value_map: ValueMap<T>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T> Aggregator<T> for DeltaSum<T>
where
    T: Number<T>,
{
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.value_map.aggregate(measurement, attrs)
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let mut values = match self.value_map.values.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        if values.is_empty() {
            return None;
        }

        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        let data_points = values
            .drain()
            .map(|(attrs, value)| DataPoint {
                attributes: attrs,
                start_time: Some(prev_start),
                time: Some(t),
                value,
                exemplars: vec![],
            })
            .collect();
        let out = data::Sum {
            temporality: Temporality::Delta,
            is_monotonic: self.monotonic,
            data_points,
        };

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

        Some(Box::new(out))
    }
}

/// Returns an [Aggregator] that summarizes a set of measurements as their
/// arithmetic sum.
///
/// Each sum is scoped by attributes and the aggregation cycle the measurements
/// were made in.
///
/// The monotonic value is used to communicate the produced [Aggregation] is
/// monotonic or not. The returned [Aggregator] does not make any guarantees this
/// value is accurate. It is up to the caller to ensure it.
///
/// Each aggregation cycle is treated independently. When the returned
/// Aggregator's Aggregation method is called it will reset all sums to zero.
pub(crate) fn new_cumulative_sum<T: Number<T>>(monotonic: bool) -> Arc<dyn Aggregator<T>> {
    Arc::new(CumulativeSum {
        value_map: ValueMap::new(),
        monotonic,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Summarizes a set of measurements made over all aggregation cycles as their
/// arithmetic sum.
struct CumulativeSum<T: Number<T>> {
    value_map: ValueMap<T>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Aggregator<T> for CumulativeSum<T> {
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.value_map.aggregate(measurement, attrs)
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let values = match self.value_map.values.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        if values.is_empty() {
            return None;
        }

        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        let data_points = values
            .iter()
            .map(|(attrs, value)| DataPoint {
                attributes: attrs.clone(),
                start_time: Some(prev_start),
                time: Some(t),
                value: *value,
                exemplars: vec![],
            })
            .collect();

        let out: data::Sum<T> = data::Sum {
            temporality: Temporality::Cumulative,
            is_monotonic: self.monotonic,
            data_points,
        };

        Some(Box::new(out))
    }
}

/// The recorded measurement value for a set of attributes.
#[derive(Default)]
struct PrecomputedValue<T: Number<T>> {
    /// The last value measured for a set of attributes that were not filtered.
    measured: T,
    /// The sum of values from measurements that had their attributes filtered.
    filtered: T,
}

/// The storage for precomputed sums.
#[derive(Default)]
struct PrecomputedMap<T: Number<T>> {
    values: Mutex<HashMap<AttributeSet, PrecomputedValue<T>>>,
}

impl<T: Number<T>> PrecomputedMap<T> {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

impl<T: Number<T>> Aggregator<T> for PrecomputedMap<T> {
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        let mut values = match self.values.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let size = values.len();
        match values.entry(attrs) {
            Entry::Occupied(mut occupied_entry) => {
                let v = occupied_entry.get_mut();
                v.measured = measurement;
            }
            Entry::Vacant(vacant_entry) => {
                if self.is_under_cardinality_limit(size) {
                    vacant_entry.insert(PrecomputedValue {
                        measured: measurement,
                        ..Default::default()
                    });
                } else {
                    values.insert(
                        get_stream_overflow_attribute_set().clone(),
                        PrecomputedValue {
                            measured: measurement,
                            ..Default::default()
                        },
                    );
                    global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                }
            }
        }
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        None // Never called
    }
}

impl<T: Number<T>> PrecomputeAggregator<T> for PrecomputedMap<T> {
    fn aggregate_filtered(&self, measurement: T, attrs: AttributeSet) {
        let mut values = match self.values.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        values
            .entry(attrs)
            .and_modify(|v| v.filtered = measurement)
            .or_insert(PrecomputedValue {
                filtered: measurement,
                ..Default::default()
            });
    }
}

/// An [Aggregator] that summarizes a set of pre-computed sums.
///
/// Each sum is scoped by attributes and the aggregation cycle the measurements
/// were made in.
///
/// The `monotonic` value is used to specify if the produced [Aggregation] is
/// monotonic or not. The returned [Aggregator] does not make any guarantees this
/// value is accurate. It is up to the caller to ensure it.
///
/// The output [Aggregation] will report recorded values as delta temporality.
pub(crate) fn new_precomputed_delta_sum<T>(monotonic: bool) -> Arc<dyn Aggregator<T>>
where
    T: Number<T>,
{
    Arc::new(PrecomputedDeltaSum {
        precomputed_map: PrecomputedMap::new(),
        reported: Default::default(),
        monotonic,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Summarizes a set of pre-computed sums recorded over all aggregation cycles
/// as the delta of these sums.
pub(crate) struct PrecomputedDeltaSum<T: Number<T>> {
    precomputed_map: PrecomputedMap<T>,
    reported: Mutex<HashMap<AttributeSet, T>>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T> Aggregator<T> for PrecomputedDeltaSum<T>
where
    T: Number<T>,
{
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.precomputed_map.aggregate(measurement, attrs)
    }

    /// Returns the recorded pre-computed sums as an [Aggregation].
    ///
    /// The sum values are expressed as the delta between what was measured this
    /// collection cycle and the previous.
    ///
    /// All pre-computed sums that were recorded for attributes sets reduced by an
    /// attribute filter (filtered-sums) are summed together and added to any
    /// pre-computed sum value recorded directly for the resulting attribute set
    /// (unfiltered-sum). The filtered-sums are reset to zero for the next
    /// collection cycle, and the unfiltered-sum is kept for the next collection
    /// cycle.
    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let mut values = match self.precomputed_map.values.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };
        let mut reported = match self.reported.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        if values.is_empty() {
            return None;
        }

        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        let data_points = values
            .iter_mut()
            .map(|(attrs, value)| {
                let v: T = value.measured.sub(value.filtered);
                let default = T::default();
                let delta = v - *reported.get(attrs).unwrap_or(&default);
                if delta != default {
                    reported.insert(attrs.clone(), v);
                }
                value.filtered = T::default();
                DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: delta,
                    exemplars: vec![],
                }
            })
            .collect();
        let out = data::Sum {
            temporality: Temporality::Delta,
            is_monotonic: self.monotonic,
            data_points,
        };

        // The delta collection cycle resets.
        let _ = self.start.lock().map(|mut start| *start = t);

        drop(reported); // drop before values guard is dropped

        Some(Box::new(out))
    }
}

/// An [Aggregator] that summarizes a set of pre-computed sums.
///
/// Each sum is scoped by attributes and the aggregation cycle the measurements
/// were made in.
///
/// The `monotonic` value is used to specify if the produced [Aggregation] is
/// monotonic or not. The returned [Aggregator] does not make any guarantees this
/// value is accurate. It is up to the caller to ensure it.
///
/// The output [Aggregation] will report recorded values as cumulative
/// temporality.
pub(crate) fn new_precomputed_cumulative_sum<T>(monotonic: bool) -> Arc<dyn Aggregator<T>>
where
    T: Number<T>,
{
    Arc::new(PrecomputedCumulativeSum {
        precomputed_map: PrecomputedMap::default(),
        monotonic,
        start: Mutex::new(SystemTime::now()),
    })
}

/// Directly records and reports a set of pre-computed sums.
pub(crate) struct PrecomputedCumulativeSum<T: Number<T>> {
    precomputed_map: PrecomputedMap<T>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T> Aggregator<T> for PrecomputedCumulativeSum<T>
where
    T: Number<T>,
{
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
        self.precomputed_map.aggregate(measurement, attrs)
    }

    /// Returns the recorded pre-computed sums as an [Aggregation].
    ///
    /// The sum values are expressed directly as they are assumed to be recorded as
    /// the cumulative sum of a some measured phenomena.
    ///
    /// All pre-computed sums that were recorded for attributes sets reduced by an
    /// attribute filter (filtered-sums) are summed together and added to any
    /// pre-computed sum value recorded directly for the resulting attribute set
    /// (unfiltered-sum). The filtered-sums are reset to zero for the next
    /// collection cycle, and the unfiltered-sum is kept for the next collection
    /// cycle.
    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        let mut values = match self.precomputed_map.values.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        if values.is_empty() {
            return None;
        }

        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        let data_points = values
            .iter_mut()
            .map(|(attrs, value)| {
                let v = value.measured + value.filtered;
                value.filtered = T::default();
                DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: v,
                    exemplars: vec![],
                }
            })
            .collect();
        let out = data::Sum {
            temporality: Temporality::Cumulative,
            is_monotonic: self.monotonic,
            data_points,
        };

        // The delta collection cycle resets.
        let _ = self.start.lock().map(|mut start| *start = t);

        Some(Box::new(out))
    }
}
