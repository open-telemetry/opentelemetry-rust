use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
    time::SystemTime,
};

use crate::{metrics::data::DataPoint};
use opentelemetry::{attributes::AttributeSet, global, metrics::MetricsError};

use super::{
    aggregate::{is_under_cardinality_limit, STREAM_OVERFLOW_ATTRIBUTE_SET},
    Number,
};

/// Timestamped measurement data.
struct DataPointValue<T> {
    timestamp: SystemTime,
    value: T,
}

/// Summarizes a set of measurements as the last one made.
#[derive(Default)]
pub(crate) struct LastValue<T> {
    values: Mutex<HashMap<AttributeSet, DataPointValue<T>>>,
}

impl<T: Number<T>> LastValue<T> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn measure(&self, measurement: T, attrs: AttributeSet) {
        let d: DataPointValue<T> = DataPointValue {
            timestamp: SystemTime::now(),
            value: measurement,
        };
        if let Ok(mut values) = self.values.lock() {
            let size = values.len();
            match values.entry(attrs) {
                Entry::Occupied(mut occupied_entry) => {
                    occupied_entry.insert(d);
                }
                Entry::Vacant(vacant_entry) => {
                    if is_under_cardinality_limit(size) {
                        vacant_entry.insert(d);
                    } else {
                        values.insert(STREAM_OVERFLOW_ATTRIBUTE_SET.clone(), d);
                        global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                    }
                }
            }
        }
    }

    pub(crate) fn compute_aggregation(&self, dest: &mut Vec<DataPoint<T>>) {
        let mut values = match self.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => {
                dest.clear(); // poisoned or no values recorded yet
                return;
            }
        };

        let n = values.len();
        if n > dest.capacity() {
            dest.reserve(n - dest.capacity());
        }

        for (i, (attrs, value)) in values.drain().enumerate() {
            if let Some(dp) = dest.get_mut(i) {
                dp.attributes = attrs;
                dp.time = Some(value.timestamp);
                dp.value = value.value;
                dp.start_time = None;
                dp.exemplars.clear();
            } else {
                dest.push(DataPoint {
                    attributes: attrs,
                    time: Some(value.timestamp),
                    value: value.value,
                    start_time: None,
                    exemplars: vec![],
                });
            }
        }

        dest.truncate(n)
    }
}
