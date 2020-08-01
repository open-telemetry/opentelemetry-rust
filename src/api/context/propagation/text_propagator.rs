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
    /// the [`Injector`].
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Injector`]: ../trait.Injector.html
    fn inject(&self, injector: &mut dyn api::Injector) {
        self.inject_context(&Context::current(), injector)
    }

    /// Properly encodes the values of the [`Context`] and injects them into the
    /// [`Injector`].
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Injector`]: ../trait.Carrier.html
    fn inject_context(&self, cx: &Context, injector: &mut dyn api::Injector);

    /// Retrieves encoded data using the provided [`Extractor`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the current
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Extractor`]: ../trait.Carrier.html
    fn extract(&self, extractor: &dyn api::Extractor) -> Context {
        self.extract_with_context(&Context::current(), extractor)
    }

    /// Retrieves encoded data using the provided [`Extractor`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the given
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Extractor`]: ../trait.Carrier.html
    fn extract_with_context(&self, cx: &Context, extractor: &dyn api::Extractor) -> Context;
}
