use std::cell::RefCell;

use opentelemetry::time::now;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use super::DroppedAttributes;
use crate::metrics::data::Exemplar;
use crate::metrics::internal::Number;
use crate::metrics::ExemplarFilter;

thread_local! {
    /// Reservoir sampling needs a uniform draw, not an unpredictable one, so a
    /// small non-cryptographic generator keeps the draw cheap while the
    /// aggregator lock is held.
    static RESERVOIR_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
}

/// A measurement that passed the [`ExemplarFilter`] and is being offered to a
/// reservoir.
///
/// An offer owns nothing and allocates nothing: it lives on the recording
/// thread's stack and only borrows what would be needed to build an
/// [`Exemplar`]. The reservoir decides first, and the exemplar — its timestamp
/// and its filtered attributes included — is materialized only for a
/// measurement the reservoir actually keeps.
#[derive(Debug)]
pub(crate) struct ExemplarOffer<'a> {
    trace_id: [u8; 16],
    span_id: [u8; 8],
    dropped: DroppedAttributes<'a>,
}

/// What an aggregator receives alongside the measured value. A single nullable
/// pointer, so an ineligible measurement — the common case — carries nothing
/// more than a null down to the aggregator.
pub(crate) type OfferRef<'a> = Option<&'a ExemplarOffer<'a>>;

impl<'a> ExemplarOffer<'a> {
    /// Borrows an offer in the shape an aggregator's `update` accepts.
    #[inline]
    pub(crate) fn by_ref(offer: &'a Option<ExemplarOffer<'a>>) -> OfferRef<'a> {
        offer.as_ref()
    }

    fn to_exemplar<T>(&self, value: T) -> Exemplar<T> {
        Exemplar {
            filtered_attributes: self.dropped.to_vec(),
            // Read while the aggregator lock is held, so the timestamp always
            // falls inside the collection interval the exemplar is exported
            // with, even when a collection races this measurement.
            time: now(),
            value,
            span_id: self.span_id,
            trace_id: self.trace_id,
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

    /// Whether any measurement can ever become an exemplar. Lets a bound
    /// instrument skip resolving its dropped attributes when none can.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    pub(crate) fn is_enabled(&self) -> bool {
        self.filter != ExemplarFilter::AlwaysOff
    }

    /// Returns an offer if this measurement is eligible to become an exemplar.
    ///
    /// `AlwaysOff` returns before touching thread-local storage, so a user who
    /// has opted out pays only a predictable branch on an enum discriminant.
    /// The eligible case copies the 24 bytes of trace and span id and nothing
    /// else: no allocation, no clock read and no clone of the span context,
    /// whose `TraceState` would otherwise be deep-copied per measurement.
    #[inline]
    pub(crate) fn offer<'a>(&self, dropped: DroppedAttributes<'a>) -> Option<ExemplarOffer<'a>> {
        match self.filter {
            ExemplarFilter::AlwaysOff => None,
            ExemplarFilter::AlwaysOn => Some(Context::map_current(|cx| {
                // AlwaysOn makes the measurement eligible independently of the
                // span's sampling decision. Preserve any active span context;
                // only an absent span leaves the ids zeroed.
                let (trace_id, span_id) = if cx.has_active_span() {
                    let span = cx.span();
                    let span_cx = span.span_context();
                    (span_cx.trace_id().to_bytes(), span_cx.span_id().to_bytes())
                } else {
                    ([0; 16], [0; 8])
                };
                ExemplarOffer {
                    trace_id,
                    span_id,
                    dropped,
                }
            })),
            ExemplarFilter::TraceBased => Context::map_current(|cx| {
                if !cx.has_active_span() {
                    return None;
                }
                let span = cx.span();
                let span_cx = span.span_context();
                if !span_cx.is_sampled() {
                    return None;
                }
                Some(ExemplarOffer {
                    trace_id: span_cx.trace_id().to_bytes(),
                    span_id: span_cx.span_id().to_bytes(),
                    dropped,
                })
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
    buckets: usize,
    /// One `(seen, exemplar)` slot per bucket, where `seen` counts the eligible
    /// measurements since the last collection and weights the sampling
    /// uniformly. Left unallocated until the first eligible measurement, so an
    /// attribute set that never records inside a sampled span costs no memory.
    slots: Vec<(u64, Option<Exemplar<T>>)>,
}

impl<T: Number> AlignedHistogramBucketReservoir<T> {
    pub(crate) fn new(buckets: usize) -> Self {
        Self {
            buckets,
            slots: Vec::new(),
        }
    }

    pub(crate) fn offer(&mut self, value: T, index: usize, offer: OfferRef<'_>) {
        let Some(offer) = offer else {
            return;
        };
        if index >= self.buckets {
            return;
        }
        if self.slots.is_empty() {
            self.slots.resize_with(self.buckets, || (0, None));
        }
        let (seen, slot) = &mut self.slots[index];
        // Reservoir sampling with k=1: the n-th measurement in this bucket
        // replaces the held exemplar with probability 1/n, which leaves every
        // measurement the bucket has seen equally likely to be the survivor.
        let keep =
            *seen == 0 || RESERVOIR_RNG.with(|rng| rng.borrow_mut().random_range(0..=*seen)) == 0;
        *seen += 1;
        if keep {
            *slot = Some(offer.to_exemplar(value));
        }
    }

    /// Drains the reservoir, returning what it held and readying it for the
    /// next collection interval.
    pub(crate) fn take(&mut self) -> Vec<Exemplar<T>> {
        self.slots
            .iter_mut()
            .filter_map(|(seen, slot)| {
                *seen = 0;
                slot.take()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer() -> Option<ExemplarOffer<'static>> {
        ExemplarSampler::new(ExemplarFilter::AlwaysOn).offer(DroppedAttributes::Resolved(&[]))
    }

    #[test]
    fn reservoir_ignores_an_out_of_range_bucket_index() {
        let mut reservoir = AlignedHistogramBucketReservoir::<i64>::new(1);

        reservoir.offer(1, 1, ExemplarOffer::by_ref(&offer()));

        assert!(reservoir.take().is_empty());
    }

    #[test]
    fn reservoir_allocates_nothing_until_a_measurement_is_eligible() {
        let mut reservoir = AlignedHistogramBucketReservoir::<i64>::new(16);

        reservoir.offer(1, 0, None);
        assert_eq!(reservoir.slots.capacity(), 0);
        assert!(reservoir.take().is_empty());

        reservoir.offer(1, 0, ExemplarOffer::by_ref(&offer()));
        assert_eq!(reservoir.take().len(), 1);
    }

    #[test]
    fn exemplar_is_stamped_when_it_is_kept() {
        let mut reservoir = AlignedHistogramBucketReservoir::<i64>::new(1);
        let offer = offer();
        let before = now();

        reservoir.offer(1, 0, ExemplarOffer::by_ref(&offer));

        assert!(reservoir.take()[0].time >= before);
    }
}
