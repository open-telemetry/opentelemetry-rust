//! # OpenTelemetry Experimental Propagator interface
//!
//! ## Binary Format
//!
//! `BinaryFormat` is a formatter to serialize and deserialize a value
//! into a binary format.
//!
//! `BinaryFormat` MUST expose the APIs that serializes values into bytes,
//! and deserializes values from bytes.
//!
//! ### ToBytes
//!
//! Serializes the given value into the on-the-wire representation.
//!
//! Required arguments:
//!
//! - the value to serialize, can be `SpanContext` or `DistributedContext`.
//!
//! Returns the on-the-wire byte representation of the value.
//!
//! ### FromBytes
//!
//! Creates a value from the given on-the-wire encoded representation.
//!
//! If the value could not be parsed, the underlying implementation
//! SHOULD decide to return ether an empty value, an invalid value, or
//! a valid value.
//!
//! Required arguments:
//!
//! - on-the-wire byte representation of the value.
//!
//! Returns a value deserialized from bytes.
//!

#[cfg(feature = "base64")]
mod base64_format;
mod binary_propagator;

#[cfg(feature = "base64")]
pub use base64_format::Base64Format;
pub use binary_propagator::{BinaryFormat, BinaryPropagator};
