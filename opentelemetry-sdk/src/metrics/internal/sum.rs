use std::vec;
use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};
use opentelemetry::KeyValue;

use super::{collect_data_points_readonly, collect_data_points_reset, AtomicTracker, Number};
use super::{Increment, ValueMap};

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number<T>> {
    value_map: ValueMap<T, T, Increment>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Sum<T> {
    /// Returns an aggregator that summarizes a set of measurements as their
    /// arithmetic sum.
    ///
    /// Each sum is scoped by attributes and the aggregation cycle the measurements
    /// were made in.
    pub(crate) fn new(monotonic: bool) -> Self {
        Sum {
            value_map: ValueMap::new(),
            monotonic,
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to Sum.
        self.value_map.measure(measurement, attrs, 0);
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let now = SystemTime::now();

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                temporality: Temporality::Delta,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.temporality = Temporality::Delta;
        s_data.is_monotonic = self.monotonic;
        s_data.data_points.clear();

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(now);

        let Ok(mut trackers) = self.value_map.trackers.write() else {
            return (0, None);
        };

        collect_data_points_reset(
            &self.value_map.no_attribs_tracker,
            &mut trackers,
            &mut s_data.data_points,
            |attributes, tracker| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(now),
                value: tracker.get_and_reset_value(),
                exemplars: vec![],
            },
        );

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = now;
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                temporality: Temporality::Cumulative,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.temporality = Temporality::Cumulative;
        s_data.is_monotonic = self.monotonic;
        s_data.data_points.clear();

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        let Ok(trackers) = self.value_map.trackers.read() else {
            return (0, None);
        };

        collect_data_points_readonly(
            &self.value_map.no_attribs_tracker,
            &trackers,
            &mut s_data.data_points,
            |attributes, tracker| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_value(),
                exemplars: vec![],
            },
        );

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
