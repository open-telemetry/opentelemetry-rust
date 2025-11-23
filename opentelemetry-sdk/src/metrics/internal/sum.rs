use crate::metrics::data::{self, AggregatedMetrics, MetricData, SumDataPoint};
use crate::metrics::Temporality;
use opentelemetry::{otel_warn, KeyValue};

use super::aggregate::{AggregateTimeInitiator, AttributeSetFilter};
use super::{Aggregator, AtomicTracker, ComputeAggregation, Measure, Number};
use super::{AtomicallyUpdate, ValueMap};

struct Increment<T>
where
    T: AtomicallyUpdate<T>,
{
    value: T::AtomicTracker,
}

impl<T> Aggregator for Increment<T>
where
    T: Number,
{
    type InitConfig = ();
    type PreComputedValue = T;

    fn create(_init: &()) -> Self {
        Self {
            value: T::new_atomic_tracker(T::default()),
        }
    }

    fn update(&self, value: T) {
        self.value.add(value)
    }

    fn clone_and_reset(&self, _: &()) -> Self {
        Self {
            value: T::new_atomic_tracker(self.value.get_and_reset_value()),
        }
    }
}

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number> {
    value_map: ValueMap<Increment<T>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    filter: AttributeSetFilter,
    monotonic: bool,
}

impl<T: Number> Sum<T> {
    /// Returns an aggregator that summarizes a set of measurements as their
    /// arithmetic sum.
    ///
    /// Each sum is scoped by attributes and the aggregation cycle the measurements
    /// were made in.
    pub(crate) fn new(
        temporality: Temporality,
        filter: AttributeSetFilter,
        monotonic: bool,
        cardinality_limit: usize,
    ) -> Self {
        Sum {
            value_map: ValueMap::new((), cardinality_limit),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            filter,
            monotonic,
        }
    }

    pub(crate) fn delta(&self, dest: Option<&mut MetricData<T>>) -> (usize, Option<MetricData<T>>) {
        let time = self.init_time.delta();
        let s_data = dest.and_then(|d| {
            if let MetricData::Sum(sum) = d {
                Some(sum)
            } else {
                None
            }
        });
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Delta,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.start_time = time.start;
        s_data.time = time.current;
        s_data.temporality = Temporality::Delta;
        s_data.is_monotonic = self.monotonic;

        self.value_map
            .collect_and_reset(&mut s_data.data_points, |attributes, aggr| SumDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (s_data.data_points.len(), new_agg.map(Into::into))
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut MetricData<T>>,
    ) -> (usize, Option<MetricData<T>>) {
        let time = self.init_time.cumulative();
        let s_data = dest.and_then(|d| {
            if let MetricData::Sum(sum) = d {
                Some(sum)
            } else {
                None
            }
        });
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Cumulative,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));

        s_data.start_time = time.start;
        s_data.time = time.current;
        s_data.temporality = Temporality::Cumulative;
        s_data.is_monotonic = self.monotonic;

        self.value_map
            .collect_readonly(&mut s_data.data_points, |attributes, aggr| SumDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (s_data.data_points.len(), new_agg.map(Into::into))
    }
}

impl<T> Measure<T> for Sum<T>
where
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        // Validate monotonic counter increment is non-negative
        if self.monotonic && measurement < T::default() {
            otel_warn!(
                name: "Counter.NegativeValue",
                message = "Counters are monotonic and can only accept non-negative values. This measurement will be dropped.",
                value = format!("{:?}", measurement)
            );
            return;
        }

        self.filter.apply(attrs, |filtered| {
            self.value_map.measure(measurement, filtered);
        })
    }
}

impl<T> ComputeAggregation for Sum<T>
where
    T: Number,
{
    fn call(&self, dest: Option<&mut AggregatedMetrics>) -> (usize, Option<AggregatedMetrics>) {
        let data = dest.and_then(|d| T::extract_metrics_data_mut(d));
        let (len, new) = match self.temporality {
            Temporality::Delta => self.delta(data),
            _ => self.cumulative(data),
        };
        (len, new.map(T::make_aggregated_metrics))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_monotonic_rejects_negative_values() {
        let sum = Sum::<f64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            true, // monotonic = true
            100,
        );

        Measure::call(&sum, 5.0, &[]);
        Measure::call(&sum, -3.0, &[]);

        let (_len, metrics) = sum.cumulative(None);
        assert!(metrics.is_some(), "Should have metrics");
        let metrics = metrics.unwrap();

        assert!(
            matches!(metrics, MetricData::Sum(_)),
            "Expected Sum metric data"
        );
        let MetricData::Sum(sum_data) = metrics else {
            unreachable!()
        };

        assert_eq!(sum_data.data_points.len(), 1);
        assert_eq!(sum_data.data_points[0].value, 5.0);
    }

    #[test]
    fn sum_non_monotonic_accepts_negative_values() {
        // Create a non-monotonic sum (up-down counter)
        let sum = Sum::<f64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            false, // monotonic = false
            100,
        );

        Measure::call(&sum, 5.0, &[]);
        Measure::call(&sum, -3.0, &[]);

        let (_len, metrics) = sum.cumulative(None);
        assert!(metrics.is_some(), "Should have metrics");
        let metrics = metrics.unwrap();

        assert!(
            matches!(metrics, MetricData::Sum(_)),
            "Expected Sum metric data"
        );
        let MetricData::Sum(sum_data) = metrics else {
            unreachable!()
        };

        assert_eq!(sum_data.data_points.len(), 1);
        // Both values should be summed: 5.0 + (-3.0) = 2.0
        assert_eq!(sum_data.data_points[0].value, 2.0);
    }

    #[test]
    fn sum_monotonic_accepts_zero() {
        let sum = Sum::<f64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            true,
            100,
        );

        Measure::call(&sum, 0.0, &[]);

        let (_len, metrics) = sum.cumulative(None);
        assert!(metrics.is_some(), "Should have metrics");
        let metrics = metrics.unwrap();

        assert!(
            matches!(metrics, MetricData::Sum(_)),
            "Expected Sum metric data"
        );
        let MetricData::Sum(sum_data) = metrics else {
            unreachable!()
        };

        assert_eq!(sum_data.data_points.len(), 1);
        assert_eq!(sum_data.data_points[0].value, 0.0);
    }

    #[test]
    fn sum_monotonic_rejects_negative_i64() {
        let sum = Sum::<i64>::new(
            Temporality::Cumulative,
            AttributeSetFilter::new(None),
            true, // monotonic = true
            100,
        );

        Measure::call(&sum, 10, &[]);
        Measure::call(&sum, -5, &[]);

        let (_len, metrics) = sum.cumulative(None);
        assert!(metrics.is_some(), "Should have metrics");
        let metrics = metrics.unwrap();

        assert!(
            matches!(metrics, MetricData::Sum(_)),
            "Expected Sum metric data"
        );
        let MetricData::Sum(sum_data) = metrics else {
            unreachable!()
        };

        assert_eq!(sum_data.data_points.len(), 1);
        assert_eq!(sum_data.data_points[0].value, 10);
    }
}
