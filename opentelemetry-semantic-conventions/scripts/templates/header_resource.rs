//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use opentelemetry::sdk;
//! use opentelemetry_semantic_conventions as semcov;
//!
//! let _tracer = opentelemetry::sdk::export::trace::stdout::new_pipeline()
//!     .with_trace_config(sdk::trace::config().with_resource(sdk::Resource::new(vec![
//!         semcov::resource::SERVICE_NAME.string("my-service"),
//!         semcov::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!     ])))
//!     .install_simple();
//! ```
