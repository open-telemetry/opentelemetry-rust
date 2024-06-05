//! The crust of the OpenTelemetry metrics SDK.
//!
//! ## Configuration
//!
//! The metrics SDK configuration is stored with each [SdkMeterProvider].
//! Configuration for [Resource]s, [View]s, and [ManualReader] or
//! [PeriodicReader] instances can be specified.
//!
//! ### Example
//!
//! ```
//! use opentelemetry::global;
//! use opentelemetry::KeyValue;
//! use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};
//!
//! // Generate SDK configuration, resource, views, etc
//! let resource = Resource::default(); // default attributes about the current process
//!
//! // Create a meter provider with the desired config
//! let meter_provider = SdkMeterProvider::builder().with_resource(resource).build();
//! global::set_meter_provider(meter_provider.clone());
//!
//! // Use the meter provider to create meter instances
//! let meter = global::meter("my_app");
//!
//! // Create instruments scoped to the meter
//! let counter = meter
//!     .u64_counter("power_consumption")
//!     .with_unit("kWh")
//!     .init();
//!
//! // use instruments to record measurements
//! counter.add(10, &[KeyValue::new("rate", "standard")]);
//!
//! // shutdown the provider at the end of the application to ensure any metrics not yet
//! // exported are flushed.
//! meter_provider.shutdown().unwrap();
//! ```
//!
//! [Resource]: crate::Resource

pub(crate) mod aggregation;
pub mod data;
pub mod exporter;
pub(crate) mod instrument;
pub(crate) mod internal;
pub(crate) mod manual_reader;
pub(crate) mod meter;
mod meter_provider;
pub(crate) mod periodic_reader;
pub(crate) mod pipeline;
pub mod reader;
pub(crate) mod view;

pub use aggregation::*;
pub use instrument::*;
pub use manual_reader::*;
pub use meter::*;
pub use meter_provider::*;
pub use periodic_reader::*;
pub use pipeline::Pipeline;
pub use view::*;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use opentelemetry::{Key, KeyValue, Value};

/// A unique set of attributes that can be used as instrument identifiers.
///
/// This must implement [Hash], [PartialEq], and [Eq] so it may be used as
/// HashMap keys and other de-duplication methods.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct AttributeSet(Vec<KeyValue>, u64);

impl From<&[KeyValue]> for AttributeSet {
    fn from(values: &[KeyValue]) -> Self {
        let mut seen_keys = HashSet::with_capacity(values.len());
        let vec = values
            .iter()
            .rev()
            .filter_map(|kv| {
                if seen_keys.insert(kv.key.clone()) {
                    Some(kv.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        AttributeSet::new(vec)
    }
}

fn calculate_hash(values: &[KeyValue]) -> u64 {
    let mut hasher = DefaultHasher::new();
    values.iter().fold(&mut hasher, |mut hasher, item| {
        item.hash(&mut hasher);
        hasher
    });
    hasher.finish()
}

impl AttributeSet {
    fn new(mut values: Vec<KeyValue>) -> Self {
        values.sort_unstable();
        let hash = calculate_hash(&values);
        AttributeSet(values, hash)
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Retains only the attributes specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&KeyValue) -> bool,
    {
        self.0.retain(|kv| f(kv));

        // Recalculate the hash as elements are changed.
        self.1 = calculate_hash(&self.0);
    }

    /// Iterate over key value pairs in the set
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.0.iter().map(|kv| (&kv.key, &kv.value))
    }
}

impl Hash for AttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1)
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use self::data::{DataPoint, HistogramDataPoint, ScopeMetrics};
    use super::*;
    use crate::metrics::data::{ResourceMetrics, Temporality};
    use crate::metrics::reader::TemporalitySelector;
    use crate::testing::metrics::InMemoryMetricsExporterBuilder;
    use crate::{runtime, testing::metrics::InMemoryMetricsExporter};
    use opentelemetry::metrics::{Counter, Meter, UpDownCounter};
    use opentelemetry::{metrics::MeterProvider as _, KeyValue};
    use rand::{rngs, Rng, SeedableRng};
    use std::borrow::Cow;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    // Run all tests in this mod
    // cargo test metrics::tests --features=testing
    // Note for all tests from this point onwards in this mod:
    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_overflow_delta() {
        // Arrange
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        // Act
        // Record measurements with A:0, A:1,.......A:1999, which just fits in the 2000 limit
        for v in 0..2000 {
            counter.add(100, &[KeyValue::new("A", v.to_string())]);
        }

        // All of the below will now go into overflow.
        counter.add(100, &[KeyValue::new("A", "foo")]);
        counter.add(100, &[KeyValue::new("A", "another")]);
        counter.add(100, &[KeyValue::new("A", "yet_another")]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        // Expecting 2001 metric points. (2000 + 1 overflow)
        assert_eq!(sum.data_points.len(), 2001);

        let data_point =
            find_datapoint_with_key_value(&sum.data_points, "otel.metric.overflow", "true")
                .expect("overflow point expected");
        assert_eq!(data_point.value, 300);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_aggregation_cumulative() {
        // Run this test with stdout enabled to see output.
        // cargo test counter_aggregation_cumulative --features=testing -- --nocapture
        counter_aggregation_helper(Temporality::Cumulative);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_aggregation_delta() {
        // Run this test with stdout enabled to see output.
        // cargo test counter_aggregation_delta --features=testing -- --nocapture
        counter_aggregation_helper(Temporality::Delta);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn histogram_aggregation_cumulative() {
        // Run this test with stdout enabled to see output.
        // cargo test histogram_aggregation_cumulative --features=testing -- --nocapture
        histogram_aggregation_helper(Temporality::Cumulative);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn histogram_aggregation_delta() {
        // Run this test with stdout enabled to see output.
        // cargo test histogram_aggregation_delta --features=testing -- --nocapture
        histogram_aggregation_helper(Temporality::Delta);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn updown_counter_aggregation_cumulative() {
        // Run this test with stdout enabled to see output.
        // cargo test updown_counter_aggregation_cumulative --features=testing -- --nocapture
        updown_counter_aggregation_helper(Temporality::Cumulative);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn updown_counter_aggregation_delta() {
        // Run this test with stdout enabled to see output.
        // cargo test updown_counter_aggregation_delta --features=testing -- --nocapture
        updown_counter_aggregation_helper(Temporality::Delta);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn gauge_aggregation() {
        // Run this test with stdout enabled to see output.
        // cargo test gauge_aggregation --features=testing -- --nocapture

        // Gauge should use last value aggregation regardless of the aggregation temporality used.
        gauge_aggregation_helper(Temporality::Delta);
        gauge_aggregation_helper(Temporality::Cumulative);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn observable_counter_aggregation_cumulative_non_zero_increment() {
        // Run this test with stdout enabled to see output.
        // cargo test observable_counter_aggregation_cumulative_non_zero_increment --features=testing -- --nocapture
        observable_counter_aggregation_helper(Temporality::Cumulative, 100, 10, 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn observable_counter_aggregation_delta_non_zero_increment() {
        // Run this test with stdout enabled to see output.
        // cargo test observable_counter_aggregation_delta_non_zero_increment --features=testing -- --nocapture
        observable_counter_aggregation_helper(Temporality::Delta, 100, 10, 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn observable_counter_aggregation_cumulative_zero_increment() {
        // Run this test with stdout enabled to see output.
        // cargo test observable_counter_aggregation_cumulative_zero_increment --features=testing -- --nocapture
        observable_counter_aggregation_helper(Temporality::Cumulative, 100, 0, 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "Aggregation bug! https://github.com/open-telemetry/opentelemetry-rust/issues/1517"]
    async fn observable_counter_aggregation_delta_zero_increment() {
        // Run this test with stdout enabled to see output.
        // cargo test observable_counter_aggregation_delta_zero_increment --features=testing -- --nocapture
        observable_counter_aggregation_helper(Temporality::Delta, 100, 0, 4);
    }

    fn observable_counter_aggregation_helper(
        temporality: Temporality,
        start: u64,
        increment: u64,
        length: u64,
    ) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        // The Observable counter reports values[0], values[1],....values[n] on each flush.
        let values: Vec<u64> = (0..length).map(|i| start + i * increment).collect();
        println!("Testing with observable values: {:?}", values);
        let values = Arc::new(values);
        let values_clone = values.clone();
        let i = Arc::new(Mutex::new(0));
        let _observable_counter = test_context
            .meter()
            .u64_observable_counter("my_observable_counter")
            .with_unit("my_unit")
            .with_callback(move |observer| {
                let mut index = i.lock().unwrap();
                if *index < values.len() {
                    observer.observe(values[*index], &[KeyValue::new("key1", "value1")]);
                    *index += 1;
                }
            })
            .init();

        for (iter, v) in values_clone.iter().enumerate() {
            test_context.flush_metrics();
            let sum = test_context.get_aggregation::<data::Sum<u64>>("my_observable_counter", None);
            assert_eq!(sum.data_points.len(), 1);
            assert!(sum.is_monotonic, "Counter should produce monotonic.");
            if let Temporality::Cumulative = temporality {
                assert_eq!(
                    sum.temporality,
                    Temporality::Cumulative,
                    "Should produce cumulative"
                );
            } else {
                assert_eq!(sum.temporality, Temporality::Delta, "Should produce delta");
            }

            // find and validate key1=value1 datapoint
            let data_point = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
                .expect("datapoint with key1=value1 expected");
            if let Temporality::Cumulative = temporality {
                // Cumulative counter should have the value as is.
                assert_eq!(data_point.value, *v);
            } else {
                // Delta counter should have the increment value.
                // Except for the first value which should be the start value.
                if iter == 0 {
                    assert_eq!(data_point.value, start);
                } else {
                    assert_eq!(data_point.value, increment);
                }
            }

            test_context.reset_metrics();
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_duplicate_instrument_merge() {
        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let counter_duplicated = meter
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let attribute = vec![KeyValue::new("key1", "value1")];
        counter.add(10, &attribute);
        counter_duplicated.add(5, &attribute);

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(
            resource_metrics[0].scope_metrics[0].metrics.len() == 1,
            "There should be single metric merging duplicate instruments"
        );
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit, "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 1 time-series.
        assert_eq!(sum.data_points.len(), 1);

        let datapoint = &sum.data_points[0];
        assert_eq!(datapoint.value, 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_duplicate_instrument_different_meter_no_merge() {
        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter1 = meter_provider.meter("test.meter1");
        let meter2 = meter_provider.meter("test.meter2");
        let counter1 = meter1
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let counter2 = meter2
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let attribute = vec![KeyValue::new("key1", "value1")];
        counter1.add(10, &attribute);
        counter2.add(5, &attribute);

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(
            resource_metrics[0].scope_metrics.len() == 2,
            "There should be 2 separate scope"
        );
        assert!(
            resource_metrics[0].scope_metrics[0].metrics.len() == 1,
            "There should be single metric for the scope"
        );
        assert!(
            resource_metrics[0].scope_metrics[1].metrics.len() == 1,
            "There should be single metric for the scope"
        );

        let scope1 = find_scope_metric(&resource_metrics[0].scope_metrics, "test.meter1");
        let scope2 = find_scope_metric(&resource_metrics[0].scope_metrics, "test.meter2");

        if let Some(scope1) = scope1 {
            let metric1 = &scope1.metrics[0];
            assert_eq!(metric1.name, "my_counter");
            assert_eq!(metric1.unit, "my_unit");
            assert_eq!(metric1.description, "my_description");
            let sum1 = metric1
                .data
                .as_any()
                .downcast_ref::<data::Sum<u64>>()
                .expect("Sum aggregation expected for Counter instruments by default");

            // Expecting 1 time-series.
            assert_eq!(sum1.data_points.len(), 1);

            let datapoint1 = &sum1.data_points[0];
            assert_eq!(datapoint1.value, 10);
        } else {
            panic!("No MetricScope found for 'test.meter1'");
        }

        if let Some(scope2) = scope2 {
            let metric2 = &scope2.metrics[0];
            assert_eq!(metric2.name, "my_counter");
            assert_eq!(metric2.unit, "my_unit");
            assert_eq!(metric2.description, "my_description");
            let sum2 = metric2
                .data
                .as_any()
                .downcast_ref::<data::Sum<u64>>()
                .expect("Sum aggregation expected for Counter instruments by default");

            // Expecting 1 time-series.
            assert_eq!(sum2.data_points.len(), 1);

            let datapoint2 = &sum2.data_points[0];
            assert_eq!(datapoint2.value, 5);
        } else {
            panic!("No MetricScope found for 'test.meter2'");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn instrumentation_scope_identity_test() {
        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        // Meters are identical except for scope attributes, but scope attributes are not an identifying property.
        // Hence there should be a single metric stream output for this test.
        let meter1 = meter_provider.versioned_meter(
            "test.meter",
            Some("v0.1.0"),
            Some("schema_url"),
            Some(vec![KeyValue::new("key", "value1")]),
        );
        let meter2 = meter_provider.versioned_meter(
            "test.meter",
            Some("v0.1.0"),
            Some("schema_url"),
            Some(vec![KeyValue::new("key", "value2")]),
        );
        let counter1 = meter1
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let counter2 = meter2
            .u64_counter("my_counter")
            .with_unit("my_unit")
            .with_description("my_description")
            .init();

        let attribute = vec![KeyValue::new("key1", "value1")];
        counter1.add(10, &attribute);
        counter2.add(5, &attribute);

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        println!("resource_metrics: {:?}", resource_metrics);
        assert!(
            resource_metrics[0].scope_metrics.len() == 1,
            "There should be a single scope as the meters are identical"
        );
        assert!(
            resource_metrics[0].scope_metrics[0].metrics.len() == 1,
            "There should be single metric for the scope as instruments are identical"
        );

        let scope = &resource_metrics[0].scope_metrics[0].scope;
        assert_eq!(scope.name, "test.meter");
        assert_eq!(scope.version, Some(Cow::Borrowed("v0.1.0")));
        assert_eq!(scope.schema_url, Some(Cow::Borrowed("schema_url")));

        // This is validating current behavior, but it is not guaranteed to be the case in the future,
        // as this is a user error and SDK reserves right to change this behavior.
        assert_eq!(scope.attributes, vec![KeyValue::new("key", "value1")]);

        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit, "my_unit");
        assert_eq!(metric.description, "my_description");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 1 time-series.
        assert_eq!(sum.data_points.len(), 1);

        let datapoint = &sum.data_points[0];
        assert_eq!(datapoint.value, 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn histogram_aggregation_with_invalid_aggregation_should_proceed_as_if_view_not_exist() {
        // Run this test with stdout enabled to see output.
        // cargo test histogram_aggregation_with_invalid_aggregation_should_proceed_as_if_view_not_exist --features=testing -- --nocapture

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let criteria = Instrument::new().name("test_histogram");
        let stream_invalid_aggregation = Stream::new()
            .aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: vec![0.9, 1.9, 1.2, 1.3, 1.4, 1.5], // invalid boundaries
                record_min_max: false,
            })
            .name("test_histogram_renamed")
            .unit("test_unit_renamed");

        let view =
            new_view(criteria, stream_invalid_aggregation).expect("Expected to create a new view");
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();

        // Act
        let meter = meter_provider.meter("test");
        let histogram = meter
            .f64_histogram("test_histogram")
            .with_unit("test_unit")
            .init();

        histogram.record(1.5, &[KeyValue::new("key1", "value1")]);
        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(
            metric.name, "test_histogram",
            "View rename should be ignored and original name retained."
        );
        assert_eq!(
            metric.unit, "test_unit",
            "View rename of unit should be ignored and original unit retained."
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn spatial_aggregation_when_view_drops_attributes_observable_counter() {
        // cargo test spatial_aggregation_when_view_drops_attributes_observable_counter --features=testing

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let criteria = Instrument::new().name("my_observable_counter");
        // View drops all attributes.
        let stream_invalid_aggregation = Stream::new().allowed_attribute_keys(vec![]);

        let view =
            new_view(criteria, stream_invalid_aggregation).expect("Expected to create a new view");
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();

        // Act
        let meter = meter_provider.meter("test");
        let _observable_counter = meter
            .u64_observable_counter("my_observable_counter")
            .with_callback(|observer| {
                observer.observe(
                    100,
                    &[
                        KeyValue::new("statusCode", "200"),
                        KeyValue::new("verb", "get"),
                    ],
                );

                observer.observe(
                    100,
                    &[
                        KeyValue::new("statusCode", "200"),
                        KeyValue::new("verb", "post"),
                    ],
                );

                observer.observe(
                    100,
                    &[
                        KeyValue::new("statusCode", "500"),
                        KeyValue::new("verb", "get"),
                    ],
                );
            })
            .init();

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_observable_counter",);

        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for ObservableCounter instruments by default");

        // Expecting 1 time-series only, as the view drops all attributes resulting
        // in a single time-series.
        // This is failing today, due to lack of support for spatial aggregation.
        assert_eq!(sum.data_points.len(), 1);

        // find and validate the single datapoint
        let data_point = &sum.data_points[0];
        assert_eq!(data_point.value, 300);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn spatial_aggregation_when_view_drops_attributes_counter() {
        // cargo test spatial_aggregation_when_view_drops_attributes_counter --features=testing

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let criteria = Instrument::new().name("my_counter");
        // View drops all attributes.
        let stream_invalid_aggregation = Stream::new().allowed_attribute_keys(vec![]);

        let view =
            new_view(criteria, stream_invalid_aggregation).expect("Expected to create a new view");
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter.u64_counter("my_counter").init();

        // Normally, this would generate 3 time-series, but since the view
        // drops all attributes, we expect only 1 time-series.
        counter.add(
            10,
            [
                KeyValue::new("statusCode", "200"),
                KeyValue::new("verb", "Get"),
            ]
            .as_ref(),
        );

        counter.add(
            10,
            [
                KeyValue::new("statusCode", "500"),
                KeyValue::new("verb", "Get"),
            ]
            .as_ref(),
        );

        counter.add(
            10,
            [
                KeyValue::new("statusCode", "200"),
                KeyValue::new("verb", "Post"),
            ]
            .as_ref(),
        );

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter",);

        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 1 time-series only, as the view drops all attributes resulting
        // in a single time-series.
        // This is failing today, due to lack of support for spatial aggregation.
        assert_eq!(sum.data_points.len(), 1);
        // find and validate the single datapoint
        let data_point = &sum.data_points[0];
        assert_eq!(data_point.value, 30);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_aggregation_attribute_order() {
        // Run this test with stdout enabled to see output.
        // cargo test counter_aggregation_attribute_order --features=testing -- --nocapture

        // Arrange
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        // Act
        // Add the same set of attributes in different order. (they are expected
        // to be treated as same attributes)
        counter.add(
            1,
            &[
                KeyValue::new("A", "a"),
                KeyValue::new("B", "b"),
                KeyValue::new("C", "c"),
            ],
        );
        counter.add(
            1,
            &[
                KeyValue::new("A", "a"),
                KeyValue::new("C", "c"),
                KeyValue::new("B", "b"),
            ],
        );
        counter.add(
            1,
            &[
                KeyValue::new("B", "b"),
                KeyValue::new("A", "a"),
                KeyValue::new("C", "c"),
            ],
        );
        counter.add(
            1,
            &[
                KeyValue::new("B", "b"),
                KeyValue::new("C", "c"),
                KeyValue::new("A", "a"),
            ],
        );
        counter.add(
            1,
            &[
                KeyValue::new("C", "c"),
                KeyValue::new("B", "b"),
                KeyValue::new("A", "a"),
            ],
        );
        counter.add(
            1,
            &[
                KeyValue::new("C", "c"),
                KeyValue::new("A", "a"),
                KeyValue::new("B", "b"),
            ],
        );
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        // Expecting 1 time-series.
        assert_eq!(sum.data_points.len(), 1);

        // validate the sole datapoint
        let data_point1 = &sum.data_points[0];
        assert_eq!(data_point1.value, 6);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_cumulative_counter() {
        let mut test_context = TestContext::new(Temporality::Cumulative);
        let counter = test_context.u64_counter("test", "my_counter", None);

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(sum.is_monotonic, "Should produce monotonic.");
        assert_eq!(
            sum.temporality,
            Temporality::Cumulative,
            "Should produce cumulative"
        );

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_delta_counter() {
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(sum.is_monotonic, "Should produce monotonic.");
        assert_eq!(sum.temporality, Temporality::Delta, "Should produce delta");

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_cumulative_up_down_counter() {
        let mut test_context = TestContext::new(Temporality::Cumulative);
        let counter = test_context.i64_up_down_counter("test", "my_counter", Some("my_unit"));

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_counter", Some("my_unit"));

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(!sum.is_monotonic, "Should not produce monotonic.");
        assert_eq!(
            sum.temporality,
            Temporality::Cumulative,
            "Should produce cumulative"
        );

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_delta_up_down_counter() {
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.i64_up_down_counter("test", "my_counter", Some("my_unit"));

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_counter", Some("my_unit"));

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(!sum.is_monotonic, "Should not produce monotonic.");
        assert_eq!(sum.temporality, Temporality::Delta, "Should produce Delta");

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_cumulative_counter_value_added_after_export() {
        let mut test_context = TestContext::new(Temporality::Cumulative);
        let counter = test_context.u64_counter("test", "my_counter", None);

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);
        test_context.reset_metrics();

        counter.add(5, &[]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(sum.is_monotonic, "Should produce monotonic.");
        assert_eq!(
            sum.temporality,
            Temporality::Cumulative,
            "Should produce cumulative"
        );

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 55, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_delta_counter_value_reset_after_export() {
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);
        test_context.reset_metrics();

        counter.add(5, &[]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(sum.is_monotonic, "Should produce monotonic.");
        assert_eq!(
            sum.temporality,
            Temporality::Delta,
            "Should produce cumulative"
        );

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 5, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn second_delta_export_does_not_give_no_attr_value_if_add_not_called() {
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);
        test_context.reset_metrics();

        counter.add(50, &[KeyValue::new("a", "b")]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        let no_attr_data_point = sum.data_points.iter().find(|x| x.attributes.is_empty());

        assert!(
            no_attr_data_point.is_none(),
            "Expected no data points with no attributes"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "Known bug: https://github.com/open-telemetry/opentelemetry-rust/issues/1598"]
    async fn delta_memory_efficiency_test() {
        // Run this test with stdout enabled to see output.
        // cargo test delta_memory_efficiency_test --features=testing -- --nocapture

        // Arrange
        let mut test_context = TestContext::new(Temporality::Delta);
        let counter = test_context.u64_counter("test", "my_counter", None);

        // Act
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);

        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);

        // find and validate key1=value1 datapoint
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.value, 5);

        // find and validate key1=value2 datapoint
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point1.value, 3);

        test_context.exporter.reset();
        // flush again, and validate that nothing is flushed
        // as delta temporality.
        test_context.flush_metrics();

        let resource_metrics = test_context
            .exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        println!("resource_metrics: {:?}", resource_metrics);
        assert!(resource_metrics.is_empty(), "No metrics should be exported as no new measurements were recorded since last collect.");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_multithreaded() {
        // Run this test with stdout enabled to see output.
        // cargo test counter_multithreaded --features=testing -- --nocapture

        counter_multithreaded_aggregation_helper(Temporality::Delta);
        counter_multithreaded_aggregation_helper(Temporality::Cumulative);
    }

    fn counter_multithreaded_aggregation_helper(temporality: Temporality) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        let counter = Arc::new(test_context.u64_counter("test", "my_counter", None));

        for i in 0..10 {
            thread::scope(|s| {
                s.spawn(|| {
                    counter.add(1, &[KeyValue::new("key1", "value1")]);
                    counter.add(1, &[KeyValue::new("key1", "value1")]);
                    counter.add(1, &[KeyValue::new("key1", "value1")]);

                    // Test concurrent collection by forcing half of the update threads to `force_flush` metrics and sleep for some time.
                    if i % 2 == 0 {
                        test_context.flush_metrics();
                        thread::sleep(Duration::from_millis(i)); // Make each thread sleep for some time duration for better testing
                    }

                    counter.add(1, &[KeyValue::new("key1", "value1")]);
                    counter.add(1, &[KeyValue::new("key1", "value1")]);
                });
            });
        }

        test_context.flush_metrics();

        // Assert
        // We invoke `test_context.flush_metrics()` six times.
        let sums =
            test_context.get_from_multiple_aggregations::<data::Sum<u64>>("my_counter", None, 6);

        let values = sums
            .iter()
            .map(|sum| {
                assert_eq!(sum.data_points.len(), 1); // Expecting 1 time-series.
                assert!(sum.is_monotonic, "Counter should produce monotonic.");
                assert_eq!(sum.temporality, temporality);

                // find and validate key1=value1 datapoint
                let data_point = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
                    .expect("datapoint with key1=value1 expected");

                data_point.value
            })
            .collect::<Vec<_>>();

        let total_sum: u64 = if temporality == Temporality::Delta {
            values.iter().sum()
        } else {
            *values.last().unwrap()
        };

        assert_eq!(total_sum, 50); // Each of the 10 update threads record measurements summing up to 5.
    }

    fn histogram_aggregation_helper(temporality: Temporality) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        let histogram = test_context.meter().u64_histogram("my_histogram").init();

        // Act
        let mut rand = rngs::SmallRng::from_entropy();
        let values_kv1 = (0..50)
            .map(|_| rand.gen_range(0..100))
            .collect::<Vec<u64>>();
        for value in values_kv1.iter() {
            histogram.record(*value, &[KeyValue::new("key1", "value1")]);
        }

        let values_kv2 = (0..30)
            .map(|_| rand.gen_range(0..100))
            .collect::<Vec<u64>>();
        for value in values_kv2.iter() {
            histogram.record(*value, &[KeyValue::new("key1", "value2")]);
        }

        test_context.flush_metrics();

        // Assert
        let histogram_data =
            test_context.get_aggregation::<data::Histogram<u64>>("my_histogram", None);
        // Expecting 2 time-series.
        assert_eq!(histogram_data.data_points.len(), 2);
        if let Temporality::Cumulative = temporality {
            assert_eq!(
                histogram_data.temporality,
                Temporality::Cumulative,
                "Should produce cumulative"
            );
        } else {
            assert_eq!(
                histogram_data.temporality,
                Temporality::Delta,
                "Should produce delta"
            );
        }

        // find and validate key1=value2 datapoint
        let data_point1 =
            find_histogram_datapoint_with_key_value(&histogram_data.data_points, "key1", "value1")
                .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.count, values_kv1.len() as u64);
        assert_eq!(data_point1.sum, values_kv1.iter().sum::<u64>());
        assert_eq!(data_point1.min.unwrap(), *values_kv1.iter().min().unwrap());
        assert_eq!(data_point1.max.unwrap(), *values_kv1.iter().max().unwrap());

        let data_point2 =
            find_histogram_datapoint_with_key_value(&histogram_data.data_points, "key1", "value2")
                .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point2.count, values_kv2.len() as u64);
        assert_eq!(data_point2.sum, values_kv2.iter().sum::<u64>());
        assert_eq!(data_point2.min.unwrap(), *values_kv2.iter().min().unwrap());
        assert_eq!(data_point2.max.unwrap(), *values_kv2.iter().max().unwrap());

        // Reset and report more measurements
        test_context.reset_metrics();
        for value in values_kv1.iter() {
            histogram.record(*value, &[KeyValue::new("key1", "value1")]);
        }

        for value in values_kv2.iter() {
            histogram.record(*value, &[KeyValue::new("key1", "value2")]);
        }

        test_context.flush_metrics();

        let histogram_data =
            test_context.get_aggregation::<data::Histogram<u64>>("my_histogram", None);
        assert_eq!(histogram_data.data_points.len(), 2);
        let data_point1 =
            find_histogram_datapoint_with_key_value(&histogram_data.data_points, "key1", "value1")
                .expect("datapoint with key1=value1 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.count, 2 * (values_kv1.len() as u64));
            assert_eq!(data_point1.sum, 2 * (values_kv1.iter().sum::<u64>()));
            assert_eq!(data_point1.min.unwrap(), *values_kv1.iter().min().unwrap());
            assert_eq!(data_point1.max.unwrap(), *values_kv1.iter().max().unwrap());
        } else {
            assert_eq!(data_point1.count, values_kv1.len() as u64);
            assert_eq!(data_point1.sum, values_kv1.iter().sum::<u64>());
            assert_eq!(data_point1.min.unwrap(), *values_kv1.iter().min().unwrap());
            assert_eq!(data_point1.max.unwrap(), *values_kv1.iter().max().unwrap());
        }

        let data_point1 =
            find_histogram_datapoint_with_key_value(&histogram_data.data_points, "key1", "value2")
                .expect("datapoint with key1=value1 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.count, 2 * (values_kv2.len() as u64));
            assert_eq!(data_point1.sum, 2 * (values_kv2.iter().sum::<u64>()));
            assert_eq!(data_point1.min.unwrap(), *values_kv2.iter().min().unwrap());
            assert_eq!(data_point1.max.unwrap(), *values_kv2.iter().max().unwrap());
        } else {
            assert_eq!(data_point1.count, values_kv2.len() as u64);
            assert_eq!(data_point1.sum, values_kv2.iter().sum::<u64>());
            assert_eq!(data_point1.min.unwrap(), *values_kv2.iter().min().unwrap());
            assert_eq!(data_point1.max.unwrap(), *values_kv2.iter().max().unwrap());
        }
    }

    fn gauge_aggregation_helper(temporality: Temporality) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        let gauge = test_context.meter().i64_gauge("my_gauge").init();

        // Act
        gauge.record(1, &[KeyValue::new("key1", "value1")]);
        gauge.record(2, &[KeyValue::new("key1", "value1")]);
        gauge.record(1, &[KeyValue::new("key1", "value1")]);
        gauge.record(3, &[KeyValue::new("key1", "value1")]);
        gauge.record(4, &[KeyValue::new("key1", "value1")]);

        gauge.record(11, &[KeyValue::new("key1", "value2")]);
        gauge.record(13, &[KeyValue::new("key1", "value2")]);
        gauge.record(6, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        // Assert
        let gauge_data_point = test_context.get_aggregation::<data::Gauge<i64>>("my_gauge", None);
        // Expecting 2 time-series.
        assert_eq!(gauge_data_point.data_points.len(), 2);

        // find and validate key1=value2 datapoint
        let data_point1 =
            find_datapoint_with_key_value(&gauge_data_point.data_points, "key1", "value1")
                .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.value, 4);

        let data_point1 =
            find_datapoint_with_key_value(&gauge_data_point.data_points, "key1", "value2")
                .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point1.value, 6);

        // Reset and report more measurements
        test_context.reset_metrics();
        gauge.record(1, &[KeyValue::new("key1", "value1")]);
        gauge.record(2, &[KeyValue::new("key1", "value1")]);
        gauge.record(11, &[KeyValue::new("key1", "value1")]);
        gauge.record(3, &[KeyValue::new("key1", "value1")]);
        gauge.record(41, &[KeyValue::new("key1", "value1")]);

        gauge.record(34, &[KeyValue::new("key1", "value2")]);
        gauge.record(12, &[KeyValue::new("key1", "value2")]);
        gauge.record(54, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Gauge<i64>>("my_gauge", None);
        assert_eq!(sum.data_points.len(), 2);
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.value, 41);

        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point1.value, 54);
    }

    fn counter_aggregation_helper(temporality: Temporality) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        let counter = test_context.u64_counter("test", "my_counter", None);

        // Act
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        // Assert
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);
        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);
        assert!(sum.is_monotonic, "Counter should produce monotonic.");
        if let Temporality::Cumulative = temporality {
            assert_eq!(
                sum.temporality,
                Temporality::Cumulative,
                "Should produce cumulative"
            );
        } else {
            assert_eq!(sum.temporality, Temporality::Delta, "Should produce delta");
        }

        // find and validate key1=value2 datapoint
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.value, 5);

        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point1.value, 3);

        // Reset and report more measurements
        test_context.reset_metrics();
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", None);
        assert_eq!(sum.data_points.len(), 2);
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.value, 10);
        } else {
            assert_eq!(data_point1.value, 5);
        }

        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.value, 6);
        } else {
            assert_eq!(data_point1.value, 3);
        }
    }

    fn updown_counter_aggregation_helper(temporality: Temporality) {
        // Arrange
        let mut test_context = TestContext::new(temporality);
        let counter = test_context.i64_up_down_counter("test", "my_updown_counter", None);

        // Act
        counter.add(10, &[KeyValue::new("key1", "value1")]);
        counter.add(-1, &[KeyValue::new("key1", "value1")]);
        counter.add(-5, &[KeyValue::new("key1", "value1")]);
        counter.add(0, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(10, &[KeyValue::new("key1", "value2")]);
        counter.add(0, &[KeyValue::new("key1", "value2")]);
        counter.add(-3, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        // Assert
        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_updown_counter", None);
        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);
        assert!(
            !sum.is_monotonic,
            "UpDownCounter should produce non-monotonic."
        );
        if let Temporality::Cumulative = temporality {
            assert_eq!(
                sum.temporality,
                Temporality::Cumulative,
                "Should produce cumulative"
            );
        } else {
            assert_eq!(sum.temporality, Temporality::Delta, "Should produce delta");
        }

        // find and validate key1=value2 datapoint
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        assert_eq!(data_point1.value, 5);

        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        assert_eq!(data_point1.value, 7);

        // Reset and report more measurements
        test_context.reset_metrics();
        counter.add(10, &[KeyValue::new("key1", "value1")]);
        counter.add(-1, &[KeyValue::new("key1", "value1")]);
        counter.add(-5, &[KeyValue::new("key1", "value1")]);
        counter.add(0, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(10, &[KeyValue::new("key1", "value2")]);
        counter.add(0, &[KeyValue::new("key1", "value2")]);
        counter.add(-3, &[KeyValue::new("key1", "value2")]);

        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_updown_counter", None);
        assert_eq!(sum.data_points.len(), 2);
        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value1")
            .expect("datapoint with key1=value1 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.value, 10);
        } else {
            assert_eq!(data_point1.value, 5);
        }

        let data_point1 = find_datapoint_with_key_value(&sum.data_points, "key1", "value2")
            .expect("datapoint with key1=value2 expected");
        if temporality == Temporality::Cumulative {
            assert_eq!(data_point1.value, 14);
        } else {
            assert_eq!(data_point1.value, 7);
        }
    }

    fn find_datapoint_with_key_value<'a, T>(
        data_points: &'a [DataPoint<T>],
        key: &str,
        value: &str,
    ) -> Option<&'a DataPoint<T>> {
        data_points.iter().find(|&datapoint| {
            datapoint
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == key && kv.value.as_str() == value)
        })
    }

    fn find_histogram_datapoint_with_key_value<'a, T>(
        data_points: &'a [HistogramDataPoint<T>],
        key: &str,
        value: &str,
    ) -> Option<&'a HistogramDataPoint<T>> {
        data_points.iter().find(|&datapoint| {
            datapoint
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == key && kv.value.as_str() == value)
        })
    }

    fn find_scope_metric<'a>(
        metrics: &'a [ScopeMetrics],
        name: &'a str,
    ) -> Option<&'a ScopeMetrics> {
        metrics
            .iter()
            .find(|&scope_metric| scope_metric.scope.name == name)
    }

    struct TestContext {
        exporter: InMemoryMetricsExporter,
        meter_provider: SdkMeterProvider,

        // Saving this on the test context for lifetime simplicity
        resource_metrics: Vec<ResourceMetrics>,
    }

    impl TestContext {
        fn new(temporality: Temporality) -> Self {
            struct TestTemporalitySelector(Temporality);
            impl TemporalitySelector for TestTemporalitySelector {
                fn temporality(&self, _kind: InstrumentKind) -> Temporality {
                    self.0
                }
            }

            let mut exporter = InMemoryMetricsExporterBuilder::new();
            exporter = exporter.with_temporality_selector(TestTemporalitySelector(temporality));

            let exporter = exporter.build();
            let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
            let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

            TestContext {
                exporter,
                meter_provider,
                resource_metrics: vec![],
            }
        }

        fn u64_counter(
            &self,
            meter_name: &'static str,
            counter_name: &'static str,
            unit: Option<&'static str>,
        ) -> Counter<u64> {
            let meter = self.meter_provider.meter(meter_name);
            let mut counter_builder = meter.u64_counter(counter_name);
            if let Some(unit_name) = unit {
                counter_builder = counter_builder.with_unit(unit_name);
            }
            counter_builder.init()
        }

        fn i64_up_down_counter(
            &self,
            meter_name: &'static str,
            counter_name: &'static str,
            unit: Option<&'static str>,
        ) -> UpDownCounter<i64> {
            let meter = self.meter_provider.meter(meter_name);
            let mut updown_counter_builder = meter.i64_up_down_counter(counter_name);
            if let Some(unit_name) = unit {
                updown_counter_builder = updown_counter_builder.with_unit(unit_name);
            }
            updown_counter_builder.init()
        }

        fn meter(&self) -> Meter {
            self.meter_provider.meter("test")
        }

        fn flush_metrics(&self) {
            self.meter_provider.force_flush().unwrap();
        }

        fn reset_metrics(&self) {
            self.exporter.reset();
        }

        fn get_aggregation<T: data::Aggregation>(
            &mut self,
            counter_name: &str,
            unit_name: Option<&str>,
        ) -> &T {
            self.resource_metrics = self
                .exporter
                .get_finished_metrics()
                .expect("metrics expected to be exported");

            assert!(
                !self.resource_metrics.is_empty(),
                "no metrics were exported"
            );

            assert!(
                self.resource_metrics.len() == 1,
                "Expected single resource metrics."
            );
            let resource_metric = self
                .resource_metrics
                .first()
                .expect("This should contain exactly one resource metric, as validated above.");

            assert!(
                !resource_metric.scope_metrics.is_empty(),
                "No scope metrics in latest export"
            );
            assert!(!resource_metric.scope_metrics[0].metrics.is_empty());

            let metric = &resource_metric.scope_metrics[0].metrics[0];
            assert_eq!(metric.name, counter_name);
            if let Some(expected_unit) = unit_name {
                assert_eq!(metric.unit, expected_unit);
            }

            metric
                .data
                .as_any()
                .downcast_ref::<T>()
                .expect("Failed to cast aggregation to expected type")
        }

        fn get_from_multiple_aggregations<T: data::Aggregation>(
            &mut self,
            counter_name: &str,
            unit_name: Option<&str>,
            invocation_count: usize,
        ) -> Vec<&T> {
            self.resource_metrics = self
                .exporter
                .get_finished_metrics()
                .expect("metrics expected to be exported");

            assert!(
                !self.resource_metrics.is_empty(),
                "no metrics were exported"
            );

            assert_eq!(
                self.resource_metrics.len(),
                invocation_count,
                "Expected collect to be called {} times",
                invocation_count
            );

            let result = self
                .resource_metrics
                .iter()
                .map(|resource_metric| {
                    assert!(
                        !resource_metric.scope_metrics.is_empty(),
                        "An export with no scope metrics occurred"
                    );

                    assert!(!resource_metric.scope_metrics[0].metrics.is_empty());

                    let metric = &resource_metric.scope_metrics[0].metrics[0];
                    assert_eq!(metric.name, counter_name);

                    if let Some(expected_unit) = unit_name {
                        assert_eq!(metric.unit, expected_unit);
                    }

                    let aggregation = metric
                        .data
                        .as_any()
                        .downcast_ref::<T>()
                        .expect("Failed to cast aggregation to expected type");
                    aggregation
                })
                .collect::<Vec<_>>();

            result
        }
    }
}
