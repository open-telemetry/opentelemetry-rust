//! # OpenTelemetry Propagator interface
//!
//! Propagators API consists of two main formats:
//!
//! - `BinaryFormat` is used to serialize and deserialize a value
//! into a binary representation.
//! - `HttpTextFormat` is used to inject and extract a value as
//! text into carriers that travel in-band across process boundaries.
//!
//! Deserializing must set `is_remote` to true on the returned
//! `SpanContext`.
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
//! ## HTTP Text Format
//!
//! `HttpTextFormat` is a formatter that injects and extracts a value
//! as text into carriers that travel in-band across process boundaries.
//!
//! Encoding is expected to conform to the HTTP Header Field semantics.
//! Values are often encoded as RPC/HTTP request headers.
//!
//! The carrier of propagated data on both the client (injector) and
//! server (extractor) side is usually an http request. Propagation is
//! usually implemented via library-specific request interceptors, where
//! the client-side injects values and the server-side extracts them.
//!
//! `HttpTextFormat` MUST expose the APIs that injects values into carriers,
//! and extracts values from carriers.
//!
//! ### Fields
//!
//! The propagation fields defined. If your carrier is reused, you should
//! delete the fields here before calling `inject`.
//!
//! For example, if the carrier is a single-use or immutable request object,
//! you don't need to clear fields as they couldn't have been set before.
//! If it is a mutable, retryable object, successive calls should clear
//! these fields first.
//!
//! The use cases of this are:
//!
//! - allow pre-allocation of fields, especially in systems like gRPC
//! Metadata
//! - allow a single-pass over an iterator
//!
//! Returns list of fields that will be used by this formatter.
//!
//! ### Inject
//!
//! Injects the value downstream. For example, as http headers.
//!
//! Required arguments:
//!
//! - the `SpanContext` to be injected.
//! - the carrier that holds propagation fields. For example, an outgoing
//! message or http request.
//! - the `Setter` invoked for each propagation key to add or remove.
//!
//! #### Setter argument
//!
//! Setter is an argument in `Inject` that puts value into given field.
//!
//! `Setter` allows a `HttpTextFormat` to set propagated fields into a
//! carrier.
//!
//! `Setter` MUST be stateless and allowed to be saved as a constant to
//! avoid runtime allocations. One of the ways to implement it is `Setter`
//! class with `Put` method as described below.
//!
//! ##### Put
//!
//! Replaces a propagated field with the given value.
//!
//! Required arguments:
//!
//! - the carrier holds propagation fields. For example, an outgoing message
//! or http request.
//! - the key of the field.
//! - the value of the field.
//!
//! ### Extract
//!
//! Extracts the value from upstream. For example, as http headers.
//!
//! If the value could not be parsed, the underlying implementation will
//! decide to return an object representing either an empty value, an invalid
//! value, or a valid value.
//!
//! Required arguments:
//!
//! - the carrier holds propagation fields. For example, an outgoing message
//! or http request.
//! - the instance of `Getter` invoked for each propagation key to get.
//!
//! Returns the non-null extracted value.
//!
//! #### Getter argument
//!
//! Getter is an argument in `Extract` that get value from given field
//!
//! `Getter` allows a `HttpTextFormat` to read propagated fields from a
//! carrier.
//!
//! `Getter` MUST be stateless and allowed to be saved as a constant to avoid
//! runtime allocations. One of the ways to implement it is `Getter` class
//! with `Get` method as described below.
//!
//! ##### Get
//!
//! Returns the first value of the given propagation key or returns `None`
//! if the key doesn't exist.
//!
//! Required arguments:
//!
//! - the carrier of propagation fields, such as an http request.
//! - the key of the field.
//!
//! Returns the first value of the given propagation key or returns `None`
//! if the key doesn't exist.
use crate::api;
use std::collections::HashMap;

/// Used to serialize and deserialize `SpanContext`s to and from  a binary
/// representation.
pub trait BinaryFormat {
    /// Serializes span context into a byte array and returns the array.
    fn to_bytes(&self, context: &api::SpanContext) -> [u8; 29];

    /// Deserializes a span context from a byte array.
    fn from_bytes(&self, bytes: Vec<u8>) -> api::SpanContext;
}

///is used to inject and extract a value as text into carriers that travel
/// in-band across process boundaries.
pub trait HttpTextFormat {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Carrier`.
    fn inject(&self, context: api::SpanContext, carrier: &mut dyn Carrier);

    /// Retrieves encoded `SpanContext`s using the `Carrier`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract(&self, carrier: &dyn Carrier) -> api::SpanContext;
}

/// Carriers provide an interface for adding and removing fields from an
/// underlying struct like `HashMap`.
pub trait Carrier {
    /// Get a value for a key from the underlying data.
    fn get(&self, key: &'static str) -> Option<&String>;
    /// Add a key and value to the underlying.
    fn set(&mut self, key: &'static str, value: String);
}

impl<S: std::hash::BuildHasher> api::Carrier for HashMap<&'static str, String, S> {
    /// Get a value for a key from the HashMap.
    fn get(&self, key: &'static str) -> Option<&String> {
        self.get(key)
    }

    /// Set a key and value in the HashMap.
    fn set(&mut self, key: &'static str, value: String) {
        self.insert(key, value);
    }
}
