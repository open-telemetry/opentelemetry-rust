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
    use tokio::time::{sleep, Duration};

    #[tokio::test]
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

        // TODO: Unable to get this to work, as
        // flush is taking forever.
        // println!("flush started");
        // meter_provider.force_flush().unwrap();
        // println!("flush finished");
        // The sleep is temporary workaround to
        // unblock progress on writing tests like
        // this.
        sleep(Duration::from_millis(100)).await;

        // Assert
        let resource_metrics = exporter
            .get_finished_metrics()
            .expect("metrics are expected to be exported.");
        assert_eq!(resource_metrics.len(), 1);
        let metric = &resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, "my_counter");
        assert_eq!(metric.unit.as_str(), "my_unit");
        let sum = metric
            .data
            .as_any()
            .downcast_ref::<data::Sum<u64>>()
            .expect("Sum aggregation expected");

        // Expecting 2 time-series.
        assert_eq!(sum.data_points.len(), 2);

        // TODO: Don't think the order is guaranteed.
        // Need to make it easy to write unit tests.
        let data_point1 = &sum.data_points[0];
        data_point1
            .attributes
            .iter()
            .find(|(k, v)| k.as_str() == "key1" && v.as_str() == "value1")
            .expect("kvp expected");
        assert_eq!(data_point1.value, 5);

        let data_point2 = &sum.data_points[1];
        data_point2
            .attributes
            .iter()
            .find(|(k, v)| k.as_str() == "key1" && v.as_str() == "value2")
            .expect("kvp expected");
        assert_eq!(data_point2.value, 3);
    }
}
