//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions
//!
//! ## Usage
//!
//! ```
//! use opentelemetry::sdk;
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let _tracer = sdk::trace::TracerProvider::builder()
//!     .with_config(sdk::trace::config().with_resource(sdk::Resource::new(vec![
//!         semconv::resource::SERVICE_NAME.string("my-service"),
//!         semconv::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!     ])))
//!     .build();
//! ```
