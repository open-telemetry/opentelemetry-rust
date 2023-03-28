use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::metrics::{
    attributes::AttributeSet,
    data::{self, Gauge},
};

use super::{Aggregator, Number};

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
        let d = DataPointValue {
            timestamp: SystemTime::now(),
            value: measurement,
        };
        let _ = self.values.lock().map(|mut values| values.insert(attrs, d));
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
