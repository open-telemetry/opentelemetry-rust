//! Utilities for working with global telemetry primitives
//!
//! ## Global Trace API
//!
//! The global trace API **provides applications access to their configured
//! [`TracerProvider`] instance from anywhere in the codebase**. This allows
//! applications to be less coupled to the specific Open Telemetry SDK while not
//! manually passing references to each part of the code that needs to create
//! [`Span`]s. Additionally, **3rd party middleware** or **library code** can be
//! written against this generic API and not constrain users to a specific
//! implementation choice.
//!
//! ### Usage in Applications
//!
//! Applications configure their tracer either by installing a trace pipeline,
//! or calling [`set_tracer_provider`].
//!
//! ```
//! # #[cfg(feature="trace")]
//! # {
//! use opentelemetry::trace::{Tracer, noop::NoopTracerProvider};
//! use opentelemetry::global;
//!
//! fn init_tracer() {
//!     // Swap this no-op provider for your tracing service of choice (jaeger, zipkin, etc)
//!     let provider = NoopTracerProvider::new();
//!
//!     // Configure the global `TracerProvider` singleton when your app starts
//!     // (there is a no-op default if this is not set by your application)
//!     let _ = global::set_tracer_provider(provider);
//! }
//!
//! fn do_something_tracked() {
//!     // Then you can get a named tracer instance anywhere in your codebase.
//!     let tracer = global::tracer("my-component");
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//! }
//!
//! // in main or other app start
//! init_tracer();
//! do_something_tracked();
//! # }
//! ```
//!
//! ### Usage in Libraries
//!
//! ```
//! # #[cfg(feature="trace")]
//! # {
//! use opentelemetry::trace::{Tracer, TracerProvider};
//! use opentelemetry::global;
//!
//! pub fn my_traced_library_function() {
//!     // End users of your library will configure their global tracer provider
//!     // so you can use the global tracer without any setup
//!     let tracer = global::tracer_provider().tracer_builder("my-library-name").
//!         with_version(env!("CARGO_PKG_VERSION")).
//!         with_schema_url("https://opentelemetry.io/schemas/1.17.0").
//!         build();
//!
//!     tracer.in_span("doing_library_work", |cx| {
//!         // Traced library logic here...
//!     });
//! }
//! # }
//! ```
//!
//! [`TracerProvider`]: crate::trace::TracerProvider
//! [`Span`]: crate::trace::Span
//!
//! ## Global Metrics API
//!
//! The global metrics API **provides applications access to their configured
//! [`MeterProvider`] instance from anywhere in the codebase**. This allows
//! applications to be less coupled to the specific Open Telemetry SDK while not
//! manually passing references to each part of the code that needs to create
//! metric instruments. Additionally, **3rd party middleware** or **library code** can be
//! written against this generic API and not constrain users to a specific
//! implementation choice.
//!
//! ### Usage in Applications
//!
//! Applications configure their meter by configuring a meter provider,
//! and calling [`set_meter_provider`] to set it as global meter provider.
//!
//! ```
//! # #[cfg(feature="metrics")]
//! # {
//! use opentelemetry::metrics::{Meter, noop::NoopMeterProvider};
//! use opentelemetry::{global, KeyValue};
//!
//! fn init_meter() {
//!     // Swap this no-op provider with an actual metrics provider,
//!     // exporting to stdout, otlp, prometheus, etc.
//!     let provider = NoopMeterProvider::new();
//!
//!     // Configure the global `MeterProvider` singleton when your app starts
//!     // (there is a no-op default if this is not set by your application)
//!     global::set_meter_provider(provider)
//! }
//!
//! fn do_something_instrumented() {
//!     // You can get a named meter instance anywhere in your codebase.
//!     let meter = global::meter("my-component");
//!     // It is recommended to reuse the same counter instance for the
//!     // lifetime of the application
//!     let counter = meter.u64_counter("my_counter").init();
//!
//!     // record measurements
//!     counter.add(1, &[KeyValue::new("mykey", "myvalue")]);
//! }
//!
//! // in main or other app start
//! init_meter();
//! do_something_instrumented();
//! // Shutdown ensures any metrics still in memory are given to exporters
//! // before the program exits.
//! shutdown_meter_provider();
//! # }
//! ```
//!
//! ### Usage in Libraries
//!
//! ```
//! # #[cfg(feature="metrics")]
//! # {
//! use opentelemetry::{global, KeyValue};
//!
//! pub fn my_instrumented_library_function() {
//!     // End users of your library will configure their global meter provider
//!     // so you can use the global meter without any setup
//!     let meter = global::meter("my-library-name");
//!     // It is recommended to reuse the same counter instance for the
//!     // lifetime of the application
//!     let counter = meter.u64_counter("my_counter").init();
//!
//!     // record measurements
//!     counter.add(1, &[KeyValue::new("mykey", "myvalue")]);
//! }
//! # }
//! ```
//!
//! [`MeterProvider`]: crate::metrics::MeterProvider
//! [`set_meter_provider`]: crate::global::set_meter_provider

mod error_handler;
#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "trace")]
mod propagation;
#[cfg(feature = "trace")]
mod trace;

pub use error_handler::{handle_error, set_error_handler, Error};
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub use metrics::*;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub use propagation::*;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub use trace::*;
