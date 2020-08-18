//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::sdk;
//! use opentelemetry_semantic_conventions as semcov;
//! use std::sync::Arc;
//!
//! let exporter = opentelemetry::exporter::trace::stdout::Builder::default().init();
//! let provider = sdk::Provider::builder()
//!    .with_simple_exporter(exporter)
//!    .with_config(sdk::Config {
//!        resource: Arc::new(sdk::Resource::new(vec![
//!            semcov::resource::SERVICE_NAME.string("my-service"),
//!            semcov::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!        ])),
//!        ..sdk::Config::default()
//!    })
//!    .build();
//! ```

use opentelemetry::api::Key;

/// Logical name of the service.
/// MUST be the same for all instances of horizontally scaled services.
pub const SERVICE_NAME: Key = Key::from_static_str("service.name");

/// A namespace for `service.name`.
/// A string value having a meaning that helps to distinguish a group of
/// services, for example the team name that owns a group of services.
/// `service.name` is expected to be unique within the same namespace. The field
/// is optional. If `service.namespace` is not specified in the Resource then
/// `service.name` is expected to be unique for all services that have no
/// explicit namespace defined (so the empty/unspecified namespace is simply one
/// more valid namespace). Zero-length namespace string is assumed equal to
/// unspecified namespace.
pub const SERVICE_NAMESPACE: Key = Key::from_static_str("service.namespace");
