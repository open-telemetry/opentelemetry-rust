use std::time::SystemTime;

use opentelemetry::time::now;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{Context, KeyValue};
use rand::Rng;

use crate::metrics::data::Exemplar;
use crate::metrics::internal::Number;
use crate::metrics::ExemplarFilter;

/// A measurement that passed the [`ExemplarFilter`] and is being offered to a
/// reservoir.
///
/// Deliberately does not carry the measured value: the value is already
/// travelling to the aggregator as part of its `PreComputedValue`, so the offer
/// only has to carry what the aggregator does not already have.
#[derive(Clone, Debug)]
pub(crate) struct ExemplarOffer {
    time: SystemTime,
    trace_id: [u8; 16],
    span_id: [u8; 8],
    /// Attributes present on the measurement but dropped by a view's attribute
    /// filter. Empty unless a view is actually filtering attributes.
    filtered_attributes: Vec<KeyValue>,
}

impl ExemplarOffer {
    fn at(time: SystemTime) -> Self {
        Self {
            time,
            trace_id: [0; 16],
            span_id: [0; 8],
            filtered_attributes: Vec::new(),
        }
    }

    /// Records the attributes that aggregation is about to discard.
    ///
    /// The spec requires an exemplar to retain any attribute from the
    /// measurement that the aggregated time series does not preserve. Without a
    /// view filter `filtered` is the same slice as `attrs` and there is nothing
    /// to record, which is the overwhelmingly common case — so the length check
    /// short-circuits before any allocation.
    pub(crate) fn set_filtered_attributes(&mut self, attrs: &[KeyValue], filtered: &[KeyValue]) {
        if attrs.len() == filtered.len() {
            return;
        }
        self.filtered_attributes = attrs
            .iter()
            .filter(|kv| !filtered.iter().any(|k| k.key == kv.key))
            .cloned()
            .collect();
    }

    fn into_exemplar<T>(self, value: T) -> Exemplar<T> {
        let this = self;
        Exemplar {
            filtered_attributes: this.filtered_attributes,
            time: this.time,
            value,
            span_id: this.span_id,
            trace_id: this.trace_id,
        }
    }
}

/// Applies an [`ExemplarFilter`] to decide whether a measurement is eligible,
/// and if so captures the ambient trace context for it.
///
/// One of these is held per instrument, so the filter is resolved once at
/// instrument creation rather than per measurement.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExemplarSampler {
    filter: ExemplarFilter,
}

impl ExemplarSampler {
    pub(crate) fn new(filter: ExemplarFilter) -> Self {
        Self { filter }
    }

    /// Returns an offer if this measurement is eligible to become an exemplar.
    ///
    /// `AlwaysOff` returns before touching thread-local storage, so a user who
    /// has opted out pays only a predictable-branch on an enum discriminant.
    ///
    /// Boxed so that the ineligible case — every measurement outside a sampled
    /// span, which is the common one — costs a null pointer to carry down to
    /// the aggregator rather than a 64-byte struct. The allocation happens only
    /// for measurements that are actually going to be sampled, which are
    /// allocating an `Exemplar` anyway.
    #[inline]
    pub(crate) fn offer(&self) -> Option<Box<ExemplarOffer>> {
        match self.filter {
            ExemplarFilter::AlwaysOff => None,
            ExemplarFilter::AlwaysOn => Some(Box::new(Context::map_current(|cx| {
                let mut offer = ExemplarOffer::at(now());
                // AlwaysOn makes the measurement eligible independently of the
                // span's sampling decision. Preserve any active span context;
                // only an absent span leaves the ids zeroed.
                if cx.has_active_span() {
                    let span_cx = cx.span().span_context().clone();
                    offer.trace_id = span_cx.trace_id().to_bytes();
                    offer.span_id = span_cx.span_id().to_bytes();
                }
                offer
            }))),
            ExemplarFilter::TraceBased => Context::map_current(|cx| {
                if !cx.has_active_span() {
                    return None;
                }
                let span_cx = cx.span().span_context().clone();
                if !span_cx.is_sampled() {
                    return None;
                }
                let mut offer = ExemplarOffer::at(now());
                offer.trace_id = span_cx.trace_id().to_bytes();
                offer.span_id = span_cx.span_id().to_bytes();
                Some(Box::new(offer))
            }),
        }
    }
}

/// `AlignedHistogramBucketExemplarReservoir` — keeps at most one exemplar per
/// explicit histogram bucket.
///
/// The bucket index is not recomputed here: the explicit-bucket histogram
/// already resolves it while precomputing its value, and hands it straight to
/// [`Self::offer`].
#[derive(Debug)]
pub(crate) struct AlignedHistogramBucketReservoir<T> {
    /// One slot per bucket. `None` until that bucket sees an eligible measurement.
    slots: Vec<Option<Exemplar<T>>>,
    /// Eligible measurements seen per bucket since the last collection, used to
    /// weight the sampling uniformly.
    seen: Vec<u64>,
}

impl<T: Number> AlignedHistogramBucketReservoir<T> {
    pub(crate) fn new(buckets: usize) -> Self {
        Self {
            slots: (0..buckets).map(|_| None).collect(),
            seen: vec![0; buckets],
        }
    }

    pub(crate) fn offer(&mut self, value: T, index: usize, offer: ExemplarOffer) {
        let Some(slot) = self.slots.get_mut(index) else {
            return;
        };
        let seen = &mut self.seen[index];
        // Reservoir sampling with k=1: the n-th measurement in this bucket
        // replaces the held exemplar with probability 1/n, which leaves every
        // measurement the bucket has seen equally likely to be the survivor.
        let keep = *seen == 0 || rand::rng().random_range(0..=*seen) == 0;
        *seen += 1;
        if keep {
            *slot = Some(offer.into_exemplar(value));
        }
    }

    /// Drains the reservoir, returning what it held and readying it for the
    /// next collection interval.
    pub(crate) fn take(&mut self) -> Vec<Exemplar<T>> {
        self.seen.iter_mut().for_each(|s| *s = 0);
        self.slots.iter_mut().filter_map(Option::take).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservoir_ignores_an_out_of_range_bucket_index() {
        let mut reservoir = AlignedHistogramBucketReservoir::<i64>::new(1);

        reservoir.offer(1, 1, ExemplarOffer::at(SystemTime::UNIX_EPOCH));

        assert!(reservoir.take().is_empty());
    }
}
