//! # Text Propagator
//!
//! `HttpTextFormat` is a formatter to serialize and deserialize a value into a
//! text format.
use crate::{api, api::Context};
use std::fmt::Debug;

/// Methods to inject and extract a value as text into carriers that travel
/// in-band across process boundaries.
pub trait HttpTextFormat: Debug {
    /// Properly encodes the values of the current [`Context`] and injects them into
    /// the [`Carrier`].
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Carrier`]: ../trait.Carrier.html
    fn inject(&self, carrier: &mut dyn api::Carrier) {
        self.inject_context(&Context::current(), carrier)
    }

    /// Properly encodes the values of the [`Context`] and injects them into the
    /// [`Carrier`].
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Carrier`]: ../trait.Carrier.html
    fn inject_context(&self, cx: &Context, carrier: &mut dyn api::Carrier);

    /// Retrieves encoded data using the provided [`Carrier`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the current
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Carrier`]: ../trait.Carrier.html
    fn extract(&self, carrier: &dyn api::Carrier) -> Context {
        self.extract_with_context(&Context::current(), carrier)
    }

    /// Retrieves encoded data using the provided [`Carrier`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the given
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Carrier`]: ../trait.Carrier.html
    fn extract_with_context(&self, cx: &Context, carrier: &dyn api::Carrier) -> Context;
}
