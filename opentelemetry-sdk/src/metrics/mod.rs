//! The rust of the OpenTelemetry metrics SDK.
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
//! use opentelemetry::{
//!     metrics::{MeterProvider, Unit},
//!     KeyValue,
//! };
//! use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};
//!
//! // Generate SDK configuration, resource, views, etc
//! let resource = Resource::default(); // default attributes about the current process
//!
//! // Create a meter provider with the desired config
//! let provider = SdkMeterProvider::builder().with_resource(resource).build();
//!
//! // Use the meter provider to create meter instances
//! let meter = provider.meter("my_app");
//!
//! // Create instruments scoped to the meter
//! let counter = meter
//!     .u64_counter("power_consumption")
//!     .with_unit(Unit::new("kWh"))
//!     .init();
//!
//! // use instruments to record measurements
//! counter.add(10, &[KeyValue::new("rate", "standard")]);
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

#[cfg(all(test, feature = "testing"))]
mod tests {
    use self::data::ScopeMetrics;
    use super::*;
    use crate::metrics::data::{ResourceMetrics, Temporality};
    use crate::metrics::reader::TemporalitySelector;
    use crate::testing::metrics::InMemoryMetricsExporterBuilder;
    use crate::{runtime, testing::metrics::InMemoryMetricsExporter};
    use opentelemetry::metrics::{Counter, UpDownCounter};
    use opentelemetry::{
        metrics::{MeterProvider as _, Unit},
        KeyValue,
    };
    use std::borrow::Cow;

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_aggregation() {
        // Run this test with stdout enabled to see output.
        // cargo test counter --features=metrics,testing -- --nocapture

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
            .init();
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);
        assert!(sum.is_monotonic, "Counter should produce monotonic.");
        assert_eq!(
            sum.temporality,
            data::Temporality::Cumulative,
            "Should produce cumulative by default."
        );

        // find and validate key1=value1 datapoint
        let mut data_point1 = None;
        for datapoint in &sum.data_points {
            if datapoint
                .attributes
                .iter()
                .any(|(k, v)| k.as_str() == "key1" && v.as_str() == "value1")
            {
                data_point1 = Some(datapoint);
            }
        }
        assert_eq!(
            data_point1
                .expect("datapoint with key1=value1 expected")
                .value,
            5
        );

        // find and validate key1=value2 datapoint
        let mut data_point1 = None;
        for datapoint in &sum.data_points {
            if datapoint
                .attributes
                .iter()
                .any(|(k, v)| k.as_str() == "key1" && v.as_str() == "value2")
            {
                data_point1 = Some(datapoint);
            }
        }
        assert_eq!(
            data_point1
                .expect("datapoint with key1=value2 expected")
                .value,
            3
        );
    }

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
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
            .with_unit(Unit::new("my_unit"))
            .with_description("my_description")
            .init();

        let counter_duplicated = meter
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
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
        assert_eq!(metric.unit.as_str(), "my_unit");
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

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
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
            .with_unit(Unit::new("my_unit"))
            .with_description("my_description")
            .init();

        let counter2 = meter2
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
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
            assert_eq!(metric1.unit.as_str(), "my_unit");
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
            assert!(false, "No MetricScope found for 'test.meter1'");
        }

        if let Some(scope2) = scope2 {
            let metric2 = &scope2.metrics[0];
            assert_eq!(metric2.name, "my_counter");
            assert_eq!(metric2.unit.as_str(), "my_unit");
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
            assert!(false, "No MetricScope found for 'test.meter2'");
        }
    }

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
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
            "test.meter1",
            Some("v0.1.0"),
            Some("schema_url"),
            Some(vec![KeyValue::new("key", "value1")]),
        );
        let meter2 = meter_provider.versioned_meter(
            "test.meter1",
            Some("v0.1.0"),
            Some("schema_url"),
            Some(vec![KeyValue::new("key", "value2")]),
        );
        let counter1 = meter1
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
            .with_description("my_description")
            .init();

        let counter2 = meter2
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
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
        assert_eq!(scope.name, "test.meter1");
        assert_eq!(scope.version, Some(Cow::Borrowed("v0.1.0")));
        assert_eq!(scope.schema_url, Some(Cow::Borrowed("schema_url")));

        // Should we validate this, as this is a user error
        assert_eq!(scope.attributes, vec![KeyValue::new("key", "value1")]);

        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
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

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn histogram_aggregation_with_invalid_aggregation_should_proceed_as_if_view_not_exist() {
        // Run this test with stdout enabled to see output.
        // cargo test histogram_aggregation_with_invalid_aggregation_should_proceed_as_if_view_not_exist --features=metrics,testing -- --nocapture

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
            .unit(Unit::new("test_unit_renamed"));

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
            .with_unit(Unit::new("test_unit"))
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
            metric.unit.as_str(),
            "test_unit",
            "View rename of unit should be ignored and original unit retained."
        );
    }

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "Spatial aggregation is not yet implemented."]
    async fn spatial_aggregation_when_view_drops_attributes_observable_counter() {
        // cargo test spatial_aggregation_when_view_drops_attributes_observable_counter --features=metrics,testing

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
        let observable_counter = meter.u64_observable_counter("my_observable_counter").init();

        // Normally, these callbacks would generate 3 time-series, but since the view
        // drops all attributes, we expect only 1 time-series.
        meter
            .register_callback(&[observable_counter.as_any()], move |observer| {
                observer.observe_u64(
                    &observable_counter,
                    100,
                    [
                        KeyValue::new("statusCode", "200"),
                        KeyValue::new("verb", "get"),
                    ]
                    .as_ref(),
                );

                observer.observe_u64(
                    &observable_counter,
                    100,
                    [
                        KeyValue::new("statusCode", "200"),
                        KeyValue::new("verb", "post"),
                    ]
                    .as_ref(),
                );

                observer.observe_u64(
                    &observable_counter,
                    100,
                    [
                        KeyValue::new("statusCode", "500"),
                        KeyValue::new("verb", "get"),
                    ]
                    .as_ref(),
                );
            })
            .expect("Expected to register callback");

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

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "Spatial aggregation is not yet implemented."]
    async fn spatial_aggregation_when_view_drops_attributes_counter() {
        // cargo test spatial_aggregation_when_view_drops_attributes_counter --features=metrics,testing

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
    async fn no_attr_cumulative_counter() {
        let mut test_context = TestContext::new(Some(Temporality::Cumulative));
        let counter = test_context.u64_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

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
        let mut test_context = TestContext::new(Some(Temporality::Delta));
        let counter = test_context.u64_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(sum.is_monotonic, "Should produce monotonic.");
        assert_eq!(sum.temporality, Temporality::Delta, "Should produce delta");

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_cumulative_up_down_counter() {
        let mut test_context = TestContext::new(Some(Temporality::Cumulative));
        let counter = test_context.i64_up_down_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_counter", "my_unit");

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
        let mut test_context = TestContext::new(Some(Temporality::Delta));
        let counter = test_context.i64_up_down_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();

        let sum = test_context.get_aggregation::<data::Sum<i64>>("my_counter", "my_unit");

        assert_eq!(sum.data_points.len(), 1, "Expected only one data point");
        assert!(!sum.is_monotonic, "Should not produce monotonic.");
        assert_eq!(sum.temporality, Temporality::Delta, "Should produce Delta");

        let data_point = &sum.data_points[0];
        assert!(data_point.attributes.is_empty(), "Non-empty attribute set");
        assert_eq!(data_point.value, 50, "Unexpected data point value");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_attr_cumulative_counter_value_added_after_export() {
        let mut test_context = TestContext::new(Some(Temporality::Cumulative));
        let counter = test_context.u64_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

        counter.add(5, &[]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

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
        let mut test_context = TestContext::new(Some(Temporality::Delta));
        let counter = test_context.u64_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

        counter.add(5, &[]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

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
        let mut test_context = TestContext::new(Some(Temporality::Delta));
        let counter = test_context.u64_counter("test", "my_counter", "my_unit");

        counter.add(50, &[]);
        test_context.flush_metrics();
        let _ = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

        counter.add(50, &[KeyValue::new("a", "b")]);
        test_context.flush_metrics();
        let sum = test_context.get_aggregation::<data::Sum<u64>>("my_counter", "my_unit");

        let no_attr_data_point = sum.data_points.iter().find(|x| x.attributes.is_empty());

        assert!(
            no_attr_data_point.is_none(),
            "Expected no data points with no attributes"
        );
    }

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "Known bug: https://github.com/open-telemetry/opentelemetry-rust/issues/1598"]
    async fn delta_memory_efficiency_test() {
        // Run this test with stdout enabled to see output.
        // cargo test delta_memory_efficiency_test --features=metrics,testing -- --nocapture

        // Arrange
        let exporter = InMemoryMetricsExporterBuilder::new()
            .with_temporality_selector(DeltaTemporalitySelector())
            .build();
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio).build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
            .init();
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);

        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);
        counter.add(1, &[KeyValue::new("key1", "value2")]);

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);
        assert!(sum.is_monotonic, "Counter should produce monotonic.");
        assert_eq!(
            sum.temporality,
            data::Temporality::Delta,
            "Should produce Delta as configured"
        );

        // find and validate key1=value1 datapoint
        let mut data_point1 = None;
        for datapoint in &sum.data_points {
            if datapoint
                .attributes
                .iter()
                .any(|(k, v)| k.as_str() == "key1" && v.as_str() == "value1")
            {
                data_point1 = Some(datapoint);
            }
        }
        assert_eq!(
            data_point1
                .expect("datapoint with key1=value1 expected")
                .value,
            5
        );

        // find and validate key1=value2 datapoint
        let mut data_point1 = None;
        for datapoint in &sum.data_points {
            if datapoint
                .attributes
                .iter()
                .any(|(k, v)| k.as_str() == "key1" && v.as_str() == "value2")
            {
                data_point1 = Some(datapoint);
            }
        }
        assert_eq!(
            data_point1
                .expect("datapoint with key1=value2 expected")
                .value,
            3
        );

        // flush again, and validate that nothing is flushed
        // as delta temporality.
        meter_provider.force_flush().unwrap();
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        println!("resource_metrics: {:?}", resource_metrics);
        assert!(resource_metrics.is_empty(), "No metrics should be exported as no new measurements were recorded since last collect.");
    }

    fn find_scope_metric<'a>(
        metrics: &'a Vec<ScopeMetrics>,
        name: &'a str,
    ) -> Option<&'a ScopeMetrics> {
        for scope_metric in metrics {
            if scope_metric.scope.name == name {
                return Some(scope_metric);
            }
        }
        None
    }

    struct DeltaTemporalitySelector();
    impl TemporalitySelector for DeltaTemporalitySelector {
        fn temporality(&self, _kind: InstrumentKind) -> Temporality {
            Temporality::Delta
        }
    }

    struct TestContext {
        exporter: InMemoryMetricsExporter,
        meter_provider: SdkMeterProvider,

        // Saving this on the test context for lifetime simplicity
        resource_metrics: Vec<ResourceMetrics>,
    }

    impl TestContext {
        fn new(temporality: Option<Temporality>) -> Self {
            struct TestTemporalitySelector(Temporality);
            impl TemporalitySelector for TestTemporalitySelector {
                fn temporality(&self, _kind: InstrumentKind) -> Temporality {
                    self.0
                }
            }

            let mut exporter = InMemoryMetricsExporterBuilder::new();
            if let Some(temporality) = temporality {
                exporter = exporter.with_temporality_selector(TestTemporalitySelector(temporality));
            }

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
            unit_name: &'static str,
        ) -> Counter<u64> {
            self.meter_provider
                .meter(meter_name)
                .u64_counter(counter_name)
                .with_unit(Unit::new(unit_name))
                .init()
        }

        fn i64_up_down_counter(
            &self,
            meter_name: &'static str,
            counter_name: &'static str,
            unit_name: &'static str,
        ) -> UpDownCounter<i64> {
            self.meter_provider
                .meter(meter_name)
                .i64_up_down_counter(counter_name)
                .with_unit(Unit::new(unit_name))
                .init()
        }

        fn flush_metrics(&self) {
            self.meter_provider.force_flush().unwrap();
        }

        fn get_aggregation<T: data::Aggregation>(
            &mut self,
            counter_name: &str,
            unit_name: &str,
        ) -> &T {
            self.resource_metrics = self
                .exporter
                .get_finished_metrics()
                .expect("metrics expected to be exported");

            assert!(
                !self.resource_metrics.is_empty(),
                "no metrics were exported"
            );

            // Get the latest resource metric in case of multiple flushes/exports
            let resource_metric = self.resource_metrics.last().unwrap();

            assert!(
                !resource_metric.scope_metrics.is_empty(),
                "No scope metrics in latest export"
            );
            assert!(!resource_metric.scope_metrics[0].metrics.is_empty());

            let metric = &resource_metric.scope_metrics[0].metrics[0];
            assert_eq!(metric.name, counter_name);
            assert_eq!(metric.unit.as_str(), unit_name);

            metric
                .data
                .as_any()
                .downcast_ref::<T>()
                .expect("Failed to cast aggregation to expected type")
        }
    }
}
