//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/resource
//!
//! ## Usage
//!
//! ```
//! use opentelemetry_sdk::{trace::{config, TracerProvider}, Resource};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let _tracer = TracerProvider::builder()
//!     .with_config(config().with_resource(sdk::Resource::new(vec![
//!         semconv::resource::SERVICE_NAME.string("my-service"),
//!         semconv::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!     ])))
//!     .build();
//! ```
