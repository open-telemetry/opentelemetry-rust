//! # OpenTelemetry Global API
//!
//! The global API **provides applications access to their configured
//! [`TracerProvider`] instance from anywhere in the codebase**. This allows
//! applications to be less coupled to the specific Open Telemetry SDK as
//! well as not manually pass references to each part of the code that needs
//! to create [`Span`]s. Additionally, **3rd party middleware** or **library code**
//! can be written against this generic API and not constrain users to a
//! specific implementation choice.
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::api::{TracerProvider, Tracer};
//! use opentelemetry::api::metrics::{Meter, MeterProvider};
//! use opentelemetry::global;
//!
//! fn init_tracer() -> global::TracerProviderGuard {
//!     let provider = opentelemetry::api::NoopProvider {};
//!
//!     // Configure the global `TracerProvider` singleton when your app starts
//!     // (there is a no-op default if this is not set by your application)
//!     global::set_tracer_provider(provider)
//! }
//!
//! fn do_something_tracked() {
//!     // Then you can use the global provider to create a tracer via `tracer`.
//!     let _span = global::tracer("my-component").start("span-name");
//!
//!     // You can also get the tracer via name and version.
//!     let _tracer = global::tracer_with_version("another-component", "1.1.1");
//!
//!     // Or access the configured provider via `tracer_provider`.
//!     let provider = global::tracer_provider();
//!     let _tracer_a = provider.get_tracer("my-component-a", None);
//!     let _tracer_b = provider.get_tracer("my-component-b", None);
//! }
//!
//! // in main or other app start
//! let _guard = init_tracer();
//! do_something_tracked();
//! ```
//!
//! ## Implementation
//!
//! This module provides types for working with the Open Telemetry API in an
//! abstract implementation-agnostic way through the use of [trait objects].
//! There is a **performance penalty** due to global synchronization as well
//! as heap allocation and dynamic dispatch (e.g. `Box<DynSpan>` vs
//! `sdk::Span`), but for many applications this overhead is likely either
//! insignificant or unavoidable as it is in the case of 3rd party integrations
//! that do not know the span type at compile time.
//!
//! ### Generic interface
//!
//! The generic interface is provided by the [`GlobalProvider`] struct which
//! can be accessed anywhere via [`tracer_provider`] and allows applications to
//! use the [`BoxedTracer`] and [`BoxedSpan`] instances that implement
//! [`Tracer`] and [`Span`]. They wrap a boxed dyn [`GenericProvider`],
//! [`GenericTracer`], and [`Span`] respectively allowing the underlying
//! implementation to be set at runtime.
//!
//! [`TracerProvider`]: ../api/trace/provider/trait.TracerProvider.html
//! [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
//! [`Span`]: ../api/trace/span/trait.Span.html
//! [`GenericProvider`]: trait.GenericProvider.html
//! [`GenericTracer`]: trait.GenericTracer.html
//! [`GlobalProvider`]: struct.GlobalProvider.html
//! [`BoxedTracer`]: struct.BoxedTracer.html
//! [`BoxedSpan`]: struct.BoxedSpan.html
//! [`tracer_provider`]: fn.tracer_provider.html
//! [trait objects]: https://doc.rust-lang.org/reference/types/trait-object.html#trait-objects

#[cfg(feature = "metrics")]
mod error_handler;
#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "trace")]
mod propagation;
#[cfg(feature = "trace")]
mod trace;

#[cfg(feature = "metrics")]
pub use error_handler::{handle_error, set_error_handler};
#[cfg(feature = "metrics")]
pub use metrics::{meter, meter_provider, set_meter_provider};
#[cfg(feature = "trace")]
pub use propagation::{get_text_map_propagator, set_text_map_propagator};
#[cfg(feature = "trace")]
pub use trace::{
    set_tracer_provider, tracer, tracer_provider, tracer_with_version, GenericProvider,
    TracerProviderGuard,
};
