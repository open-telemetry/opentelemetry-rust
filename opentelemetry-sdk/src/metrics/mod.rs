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
    use super::*;
    use crate::{runtime, testing::metrics::InMemoryMetricsExporter};
    use opentelemetry::{
        metrics::{MeterProvider as _, Unit},
        KeyValue,
    };

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
        let stream_invalid_aggregation = Stream::new()
             .allowed_attribute_keys(vec![]);

        let view =
            new_view(criteria, stream_invalid_aggregation).expect("Expected to create a new view");
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();

        // Act
        let meter = meter_provider.meter("test");
        let observable_counter = meter
        .u64_observable_counter("my_observable_counter")
        .init();

        // Normally, these callbacks would generate 3 time-series, but since the view
        // drops all attributes, we expect only 1 time-series.
        meter.register_callback(&[observable_counter.as_any()], move |observer| {
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
        }).expect("Expected to register callback");

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert!(!resource_metrics.is_empty());
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(
            metric.name, "my_observable_counter",
        );

        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for ObservableCounter instruments by default");

        // Expecting 1 time-series only, as the view drops all attributes resulting
        // in a single time-series.
        // This is failing today, due to lack of support for spatial aggregation.
        assert_eq!(sum.data_points.len(), 1);
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
        let stream_invalid_aggregation = Stream::new()
             .allowed_attribute_keys(vec![]);

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
        assert_eq!(
            metric.name, "my_counter",
        );

        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 1 time-series only, as the view drops all attributes resulting
        // in a single time-series.
        // This is failing today, due to lack of support for spatial aggregation.
        assert_eq!(sum.data_points.len(), 1);
    }
}
