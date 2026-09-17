//! Zero-sized stand-ins used when `spec_unstable_metrics_exemplars` is off.
//!
//! Every type here is a ZST and every method is an empty inlined body, so the
//! aggregators can carry a reservoir and offer to it unconditionally without
//! costing a build that does not want exemplars anything at all.

use std::marker::PhantomData;

use super::DroppedAttributes;
use crate::metrics::data::Exemplar;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExemplarOffer<'a>(PhantomData<&'a ()>);

/// Zero-sized, so a histogram's precomputed value stays exactly as wide as it
/// was before exemplars existed.
pub(crate) type OfferRef<'a> = PhantomData<&'a ()>;

impl<'a> ExemplarOffer<'a> {
    #[inline]
    pub(crate) fn by_ref(_offer: &'a Option<ExemplarOffer<'a>>) -> OfferRef<'a> {
        PhantomData
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExemplarSampler;

impl ExemplarSampler {
    #[inline]
    pub(crate) fn new() -> Self {
        Self
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    #[inline]
    pub(crate) fn is_enabled(&self) -> bool {
        false
    }

    #[inline]
    pub(crate) fn offer<'a>(&self, _dropped: DroppedAttributes<'a>) -> Option<ExemplarOffer<'a>> {
        None
    }
}

#[derive(Debug)]
pub(crate) struct AlignedHistogramBucketReservoir<T>(PhantomData<T>);

impl<T> AlignedHistogramBucketReservoir<T> {
    #[inline]
    pub(crate) fn new(_buckets: usize) -> Self {
        Self(PhantomData)
    }

    #[inline]
    pub(crate) fn offer(&mut self, _value: T, _index: usize, _offer: OfferRef<'_>) {}

    #[inline]
    pub(crate) fn take(&mut self) -> Vec<Exemplar<T>> {
        Vec::new()
    }
}
