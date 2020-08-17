//! # Text Propagator
//!
//! `HttpTextFormat` is a formatter to serialize and deserialize a value into a
//! text format.
use crate::{api, api::Context};
use std::fmt::Debug;
use std::slice;

/// Methods to inject and extract a value as text into injectors and extractors that travel
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
    /// [`Injector`]: ../trait.Injector.html
    fn inject_context(&self, cx: &Context, injector: &mut dyn api::Injector);

    /// Retrieves encoded data using the provided [`Extractor`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the current
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Extractor`]: ../trait.Extractor.html
    fn extract(&self, extractor: &dyn api::Extractor) -> Context {
        self.extract_with_context(&Context::current(), extractor)
    }

    /// Retrieves encoded data using the provided [`Extractor`]. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the given
    /// [`Context`] is returned.
    ///
    /// [`Context`]: ../../struct.Context.html
    /// [`Extractor`]: ../trait.Extractor.html
    fn extract_with_context(&self, cx: &Context, extractor: &dyn api::Extractor) -> Context;

    /// Returns iter of fields used by [`HttpTextFormat`]
    ///
    /// [`HttpTextFormat`]: ./trait.HttpTextFormat.html
    fn get_field_iter(&self) -> FieldIter;
}

/// An iterator over fields of a [`HttpTextFormat`]
///
/// [`HttpTextFormat`]: ./trait.HttpTextFormat.html
#[derive(Debug)]
pub struct FieldIter<'a>(slice::Iter<'a, String>);

impl<'a> FieldIter<'a> {
    /// Create a new `FieldIter` from a slice of propagator fields
    pub fn new(fields: &'a [String]) -> Self {
        FieldIter(fields.iter())
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|field| field.as_str())
    }
}