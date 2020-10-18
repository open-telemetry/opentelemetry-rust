//! ## OpenTelemetry API: What applications use and SDKs implement.
//!
//! OpenTelemetry Language Libraries are composed of 2 packages: `api` and `sdk`.
//!
//! Third-party libraries and frameworks that want to be instrumented in OpenTelemetry-compatible
//! way will have a dependency on the `api` package. The developers of these third-party libraries
//! will add calls to telemetry API to produce telemetry data.
//!
//! Applications that use third-party libraries that are instrumented with OpenTelemetry API will
//! have a choice to enable or not enable the actual delivery of telemetry data. The application
//! can also call telemetry API directly to produce additional telemetry data.
//!
//! In order to enable telemetry the application must take a dependency on the OpenTelemetry SDK,
//! which implements the delivery of the telemetry. The application must also configure exporters
//! so that the SDK knows where and how to deliver the telemetry.
mod baggage;
mod context;
mod core;
pub mod labels;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod propagation;
#[cfg(feature = "trace")]
pub mod trace;

pub use self::baggage::{AddBaggage, Baggage, BaggageExt};
pub use self::context::Context;
pub use self::core::{Key, KeyValue, KeyValueMetadata, Metadata, Unit, Value};
