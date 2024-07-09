//! # Metric Semantic Conventions
//!
//! The [metric semantic conventions] define a set of standardized attributes to
//! be used in `Meter`s.
//!
//! [metric semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/metric
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::global;
//! use opentelemetry::KeyValue;
//! use opentelemetry_semantic_conventions as semconv;
//!
//! // Assuming an already initialized `MeterProvider`
//! // See: https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/metrics-basic/src/main.rs
//! // for an example on how to initialize a `MeterProvider`
//! let meter = global::meter("mylibraryname");
//! let meter = provider.get("example-meter");
//! let histogram = meter
//!     .u64_histogram(semconv::metric::HTTP_SERVER_REQUEST_DURATION)
//!     .with_unit(Unit::new("By"))
//!     .with_description("Duration of HTTP server requests.")
//!     .init();
//! ```
