//! *Compiler support: [requires `rustc` 1.64+][msrv]*
//!
//! [Jaeger Docs]: https://www.jaegertracing.io/docs/
//! [jaeger-deprecation]: https://github.com/open-telemetry/opentelemetry-specification/pull/2858/files
//! [jaeger-otlp]: https://www.jaegertracing.io/docs/1.38/apis/#opentelemetry-protocol-stable
//! [otlp-exporter]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/
//! [msrv]: #supported-rust-versions
//! [jaeger propagation format]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
//!
//! # Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.64. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//!
//! The current stable Rust compiler and the three most recent minor versions
//! before it will always be supported. For example, if the current stable
//! compiler version is 1.64, the minimum supported version will not be
//! increased past 1.46, three minor versions prior. Increasing the minimum
//! supported compiler version is not considered a semver breaking change as
//! long as doing so complies with this policy.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

/// The Jaeger propagator propagates span contexts in [Jaeger propagation format].
///
/// Cross-cutting concerns send their state to the next process using `Propagator`s,
/// which are defined as objects used to read and write context data to and from messages
/// exchanged by the applications. Each concern creates a set of `Propagator`s for every
/// supported `Propagator` type.
///
/// Note that a jaeger header can be set in http header or encoded as url.
///
/// ## Examples
/// ```
/// # use opentelemetry::{global, trace::{Tracer, TraceContextExt}, Context};
/// # use opentelemetry_jaeger_propagator::Propagator as JaegerPropagator;
/// # fn send_request() {
/// // setup jaeger propagator
/// global::set_text_map_propagator(JaegerPropagator::default());
/// // You also can init propagator with custom header name
/// // global::set_text_map_propagator(JaegerPropagator::with_custom_header("my-custom-header"));
///
/// // before sending requests to downstream services.
/// let mut headers = std::collections::HashMap::new(); // replace by http header of the outgoing request
/// let caller_span = global::tracer("caller").start("say hello");
/// let cx = Context::current_with_span(caller_span);
/// global::get_text_map_propagator(|propagator| {
///     propagator.inject_context(&cx, &mut headers); // propagator serialize the tracing context
/// });
/// // Send the request..
/// # }
///
///
/// # fn receive_request() {
/// // Receive the request sent above on the other service...
/// // setup jaeger propagator
/// global::set_text_map_propagator(JaegerPropagator::new());
/// // You also can init propagator with custom header name
/// // global::set_text_map_propagator(JaegerPropagator::with_custom_header("my-custom-header"));
///
/// let headers = std::collections::HashMap::new(); // replace this with http header map from incoming requests.
/// let parent_context = global::get_text_map_propagator(|propagator| {
///      propagator.extract(&headers)
/// });
///
/// // this span's parent span will be caller_span in send_request functions.
/// let receiver_span = global::tracer("receiver").start_with_context("hello", &parent_context);
/// # }
/// ```
///
///  [jaeger propagation format]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
pub mod propagator;
#[cfg(feature = "integration_test")]
#[doc(hidden)]
pub mod testing;

pub use propagator::Propagator;
