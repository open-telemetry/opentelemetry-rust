use crate::export::metrics::aggregation::AggregationKind;
use crate::metrics::sdk_api::{Descriptor, InstrumentKind};

#[derive(Clone)]
struct ConstantTemporalitySelector(Temporality);

impl TemporalitySelector for ConstantTemporalitySelector {
    fn temporality_for(&self, _descriptor: &Descriptor, _kind: &AggregationKind) -> Temporality {
        self.0
    }
}

/// Returns an [`TemporalitySelector`] that returns a constant [`Temporality`].
pub fn constant_temporality_selector(temporality: Temporality) -> impl TemporalitySelector + Clone {
    ConstantTemporalitySelector(temporality)
}

/// Returns an [`TemporalitySelector`] that always returns [`Temporality::Cumulative`].
pub fn cumulative_temporality_selector() -> impl TemporalitySelector + Clone {
    constant_temporality_selector(Temporality::Cumulative)
}

/// Returns an [`TemporalitySelector`] that always returns [`Temporality::Delta`].
pub fn delta_temporality_selector() -> impl TemporalitySelector + Clone {
    constant_temporality_selector(Temporality::Delta)
}

/// Returns a [`TemporalitySelector`] that always returns the cumulative [`Temporality`] to avoid
/// long-term memory requirements.
pub fn stateless_temporality_selector() -> impl TemporalitySelector + Clone {
    constant_temporality_selector(Temporality::Cumulative)
}

#[derive(Clone)]
struct StatelessTemporalitySelector;

impl TemporalitySelector for StatelessTemporalitySelector {
    fn temporality_for(&self, descriptor: &Descriptor, kind: &AggregationKind) -> Temporality {
        if kind == &AggregationKind::SUM && descriptor.instrument_kind().precomputed_sum() {
            Temporality::Cumulative
        } else {
            Temporality::Delta
        }
    }
}

/// Temporality indicates the temporal aggregation exported by an exporter.
/// These bits may be OR-d together when multiple exporters are in use.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Temporality {
    /// Indicates that an Exporter expects a Cumulative Aggregation.
    Cumulative = 1,

    /// Indicates that an Exporter expects a Delta Aggregation.
    Delta = 2,
}

impl Temporality {
    /// Tests whether `kind` includes a specific kind of exporter.
    pub fn includes(&self, other: &Self) -> bool {
        (*self as u32) & (*other as u32) != 0
    }

    /// Returns whether a temporality of this kind requires memory to export correctly.
    pub fn memory_required(&self, kind: &InstrumentKind) -> bool {
        match kind {
            InstrumentKind::Histogram
            | InstrumentKind::GaugeObserver
            | InstrumentKind::Counter
            | InstrumentKind::UpDownCounter => {
                // Cumulative-oriented instruments:
                self.includes(&Temporality::Cumulative)
            }

            InstrumentKind::CounterObserver | InstrumentKind::UpDownCounterObserver => {
                // Delta-oriented instruments:
                self.includes(&Temporality::Delta)
            }
        }
    }
}

/// TemporalitySelector is a sub-interface of Exporter used to indicate
/// whether the Processor should compute Delta or Cumulative
/// Aggregations.
pub trait TemporalitySelector {
    /// TemporalityFor should return the correct Temporality that
    /// should be used when exporting data for the given metric
    /// instrument and Aggregator kind.
    fn temporality_for(&self, descriptor: &Descriptor, kind: &AggregationKind) -> Temporality;
}
