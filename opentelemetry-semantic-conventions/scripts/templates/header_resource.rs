//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/resource
//!
//! ## Usage
//!
//! `tracing`:
//! 
//! ```
//! use opentelemetry_semantic_conventions as semconv;
//! use tracing::span;
//!
//! let span = span!(
//!     LEVEL::INFO,
//!     "handle_request",
//!     { semconv::resource::SERVICE_NAME = "my-service" },
//!     { semconv::resource::SERVICE_NAMESPACE = "my-namespace" }
//! );
//! ```
//! 
//! OpenTelemetry SDK:
//! 
//! ```
//! use opentelemetry::KeyValue;
//! use opentelemetry_sdk::{trace::{config, TracerProvider}, Resource};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let _tracer = TracerProvider::builder()
//!     .with_config(config().with_resource(Resource::new(vec![
//!         KeyValue::new(semconv::resource::SERVICE_NAME, "my-service"),
//!         KeyValue::new(semconv::resource::SERVICE_NAMESPACE, "my-namespace"),
//!     ])))
//!     .build();
//! ```
