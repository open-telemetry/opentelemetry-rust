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
    use crate::metrics::data::{ResourceMetrics, Temporality};
    use crate::metrics::reader::TemporalitySelector;
    use crate::testing::metrics::InMemoryMetricsExporterBuilder;
    use crate::{runtime, testing::metrics::InMemoryMetricsExporter};
    use opentelemetry::metrics::{Counter, UpDownCounter};
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
    async fn counter_aggregation_overflow() {
        // Run this test with stdout enabled to see output.
        // cargo test counter --features=metrics,testing -- --nocapture

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        // PeriodicReader with large interval to avoid auto-flush
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio)
            .with_interval(std::time::Duration::from_secs(100000))
            .build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
            .init();

        // sleep for random ~5 milis to avoid recording during first collect cycle
        // (TBD: need to fix PeriodicReader to NOT collect data immediately after start)
        std::thread::sleep(std::time::Duration::from_millis(5));
        let unique_measurements = 1999;
        let overflow_measurements = 4;
        // Generate measurements to enforce overflow
        for i in 0..unique_measurements + overflow_measurements {
            let attribute_value = format!("value{}", i); // Creates a unique attribute value for each measurement
            counter.add(1, &[KeyValue::new("key1", attribute_value)]);
        }
        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        // Every collect cycle produces a new ResourceMetrics (even if no data is collected).
        // TBD = This needs to be fixed, and then below assert should validate for one entry
        assert!(resource_metrics.len() == 2);
        let metric = &resource_metrics[1].scope_metrics[0].metrics[0]; // second ResourceMetrics
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 2000 unique time-series.
        assert_eq!(sum.data_points.len(), unique_measurements + 1); // all overflow measurements are merged into one
        assert!(sum.is_monotonic, "Counter should produce monotonic.");
        assert_eq!(
            sum.temporality,
            data::Temporality::Cumulative,
            "Should produce cumulative by default."
        );
        // ensure that overflow attribute is persent
        for data_point in &sum.data_points {
            let mut overflow_attribute_present = false;
            for attribute in data_point.attributes.iter() {
                if attribute.0 == &opentelemetry::Key::from("otel.metric.overflow") {
                    overflow_attribute_present = true;
                    break;
                }
            }
            if overflow_attribute_present {
                assert_eq!(data_point.value, overflow_measurements as u64);
            } else {
                assert_eq!(data_point.value, 1);
            }
        }
    }

    // "multi_thread" tokio flavor must be used else flush won't
    // be able to make progress!
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_aggregation_concurrent_overflow() {
        // Run this test with stdout enabled to see output.
        // cargo test counter --features=metrics,testing -- --nocapture

        // Arrange
        let exporter = InMemoryMetricsExporter::default();
        // PeriodicReader with large interval to avoid auto-flush
        let reader = PeriodicReader::builder(exporter.clone(), runtime::Tokio)
            .with_interval(std::time::Duration::from_secs(100000))
            .build();
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();

        // Act
        let meter = meter_provider.meter("test");
        let counter = meter
            .u64_counter("my_counter")
            .with_unit(Unit::new("my_unit"))
            .init();

        // sleep for random ~5 milis to avoid recording during first collect cycle
        // (TBD: need to fix PeriodicReader to NOT collect data immediately after start)
        std::thread::sleep(std::time::Duration::from_millis(5));

        let unique_measurements = 1999;
        let overflow_measurements = 4;
        let total_measurements = unique_measurements + overflow_measurements;

        let counter = std::sync::Arc::new(std::sync::Mutex::new(counter)); // Shared counter among threads

        let num_threads = 4;
        let measurements_per_thread = total_measurements / num_threads;
        let remainder = total_measurements % num_threads; // Remainder to be added to the last thread

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let counter_clone = std::sync::Arc::clone(&counter);
            let start_index = thread_id * measurements_per_thread;
            let end_index = if thread_id == num_threads - 1 {
                start_index + measurements_per_thread + remainder // Add remainder to the last thread
            } else {
                start_index + measurements_per_thread
            };

            let handle = std::thread::spawn(move || {
                for i in start_index..end_index {
                    let attribute_value = format!("value{}", i);
                    let kv = vec![KeyValue::new("key1", attribute_value)];

                    let counter = counter_clone.lock().unwrap();
                    counter.add(1, &kv);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        meter_provider.force_flush().unwrap();

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        // Every collect cycle produces a new ResourceMetrics (even if no data is collected).
        // TBD = This needs to be fixed, and then below assert should validate for one entry
        assert!(resource_metrics.len() == 2);
        let metric = &resource_metrics[1].scope_metrics[0].metrics[0]; // second ResourceMetrics
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected for Counter instruments by default");

        // Expecting 2000 unique time-series.
        assert_eq!(sum.data_points.len(), unique_measurements + 1); // all overflow measurements are merged into one
        assert!(sum.is_monotonic, "Counter should produce monotonic.");
        assert_eq!(
            sum.temporality,
            data::Temporality::Cumulative,
            "Should produce cumulative by default."
        );

        // ensure that overflow attribute is persent
        for data_point in &sum.data_points {
            let mut overflow_attribute_present = false;
            for attribute in data_point.attributes.iter() {
                if attribute.0 == &opentelemetry::Key::from("otel.metric.overflow") {
                    overflow_attribute_present = true;
                    break;
                }
            }
            if overflow_attribute_present {
                assert_eq!(data_point.value, overflow_measurements as u64);
            } else {
                assert_eq!(data_point.value, 1);
            }
        }
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
