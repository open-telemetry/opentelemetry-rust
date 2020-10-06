//! # OpenTelemetry Propagators
//!
//! Cross-cutting concerns send their state to the next process using Propagators, which are defined
//! as objects used to read and write context data to and from messages exchanged by the
//! applications. Each concern creates a set of Propagators for every supported Propagator type. For
//! more information see the [OpenTelemetry Spec]['spec'].
//!
//! ['spec']: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/context/api-propagators.md#overview
pub(super) mod aws;
pub(super) mod b3;
pub(super) mod jaeger;
pub(super) mod w3c;
