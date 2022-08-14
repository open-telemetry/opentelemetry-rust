/// Kinds of OpenTelemetry metric instruments
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstrumentKind {
    /// A histogram instrument
    Histogram,
    /// A gauge observer instrument
    GaugeObserver,
    /// A synchronous per-request part of a monotonic sum.
    Counter,
    /// A synchronous per-request part of a non-monotonic sum.
    UpDownCounter,
    /// An asynchronous per-interval recorder of a monotonic sum.
    CounterObserver,
    /// An asynchronous per-interval recorder of a non-monotonic sum.
    UpDownCounterObserver,
}

impl InstrumentKind {
    /// Whether this is a synchronous kind of instrument.
    pub fn synchronous(&self) -> bool {
        matches!(
            self,
            InstrumentKind::Counter | InstrumentKind::UpDownCounter | InstrumentKind::Histogram
        )
    }

    /// Whether this is a synchronous kind of instrument.
    pub fn asynchronous(&self) -> bool {
        !self.synchronous()
    }

    /// Whether this kind of instrument adds its inputs (as opposed to grouping).
    pub fn adding(&self) -> bool {
        matches!(
            self,
            InstrumentKind::Counter
                | InstrumentKind::UpDownCounter
                | InstrumentKind::CounterObserver
                | InstrumentKind::UpDownCounterObserver
        )
    }

    /// Whether this kind of instrument groups its inputs (as opposed to adding).
    pub fn grouping(&self) -> bool {
        !self.adding()
    }

    /// Whether this kind of instrument exposes a non-decreasing sum.
    pub fn monotonic(&self) -> bool {
        matches!(
            self,
            InstrumentKind::Counter | InstrumentKind::CounterObserver
        )
    }

    /// Whether this kind of instrument receives precomputed sums.
    pub fn precomputed_sum(&self) -> bool {
        self.adding() && self.asynchronous()
    }
}
