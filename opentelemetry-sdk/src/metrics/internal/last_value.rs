use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::attributes::AttributeSet;
use crate::metrics::data::{self, Gauge};
use opentelemetry::{global, metrics::MetricsError};

use super::{aggregator::STREAM_OVERFLOW_ATTRIBUTE_SET, Aggregator, Number};

/// Timestamped measurement data.
struct DataPointValue<T> {
    timestamp: SystemTime,
    value: T,
}

/// An Aggregator that summarizes a set of measurements as the last one made.
pub(crate) fn new_last_value<T: Number<T>>() -> Arc<dyn Aggregator<T>> {
    Arc::new(LastValue::default())
}

/// Summarizes a set of measurements as the last one made.
#[derive(Default)]
struct LastValue<T> {
    values: Mutex<HashMap<AttributeSet, DataPointValue<T>>>,
}

impl<T: Number<T>> Aggregator<T> for LastValue<T> {
    fn aggregate(&self, measurement: T, attrs: AttributeSet) {
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
                    if self.is_under_cardinality_limit(size) {
                        vacant_entry.insert(d);
                    } else {
                        values.insert(STREAM_OVERFLOW_ATTRIBUTE_SET.clone(), d);
                        global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
                    }
                }
            }
        }
    }

    fn aggregation(&self) -> Option<Box<dyn crate::metrics::data::Aggregation>> {
        let mut values = match self.values.lock() {
            Ok(guard) if !guard.is_empty() => guard,
            _ => return None,
        };

        let data_points = values
            .drain()
            .map(|(attrs, value)| data::DataPoint {
                attributes: attrs,
                time: Some(value.timestamp),
                value: value.value,
                start_time: None,
                exemplars: vec![],
            })
            .collect();

        Some(Box::new(Gauge { data_points }))
    }
}
