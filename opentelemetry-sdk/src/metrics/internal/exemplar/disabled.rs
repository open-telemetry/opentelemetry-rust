//! Zero-sized stand-ins used when `spec_unstable_metrics_exemplars` is off.
//!
//! Every type here is a ZST and every method is an empty inlined body, so the
//! aggregators can carry a reservoir and offer to it unconditionally without
//! costing a build that does not want exemplars anything at all.

use std::marker::PhantomData;

use opentelemetry::KeyValue;

use crate::metrics::data::Exemplar;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExemplarOffer;

impl ExemplarOffer {
    #[inline]
    pub(crate) fn set_filtered_attributes(&mut self, _attrs: &[KeyValue], _filtered: &[KeyValue]) {}
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExemplarSampler;

impl ExemplarSampler {
    #[inline]
    pub(crate) fn new() -> Self {
        Self
    }

    /// Always `None`, so no aggregator ever reaches its reservoir.
    #[inline]
    pub(crate) fn offer(&self) -> Option<ExemplarOffer> {
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
    pub(crate) fn offer(&mut self, _value: T, _index: usize, _offer: ExemplarOffer) {}

    #[inline]
    pub(crate) fn take(&mut self) -> Vec<Exemplar<T>> {
        Vec::new()
    }
}
