use opentelemetry::{
    metrics::{BoundSyncInstrument, InstrumentProvider, SyncInstrument},
    KeyValue,
};

/// A no-op instance of a `Meter`
#[derive(Debug, Default)]
pub(crate) struct NoopMeter {
    _private: (),
}

impl NoopMeter {
    /// Create a new no-op meter core.
    pub(crate) fn new() -> Self {
        NoopMeter { _private: () }
    }
}

impl InstrumentProvider for NoopMeter {}

/// A no-op sync instrument
#[derive(Debug, Default)]
pub(crate) struct NoopSyncInstrument {
    _private: (),
}

impl NoopSyncInstrument {
    /// Create a new no-op sync instrument
    pub(crate) fn new() -> Self {
        NoopSyncInstrument { _private: () }
    }
}

impl<T> SyncInstrument<T> for NoopSyncInstrument {
    fn measure(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }

    fn bind(&self, _attributes: &[KeyValue]) -> Box<dyn BoundSyncInstrument<T>> {
        Box::new(NoopBoundSyncInstrument { _private: () })
    }
}

struct NoopBoundSyncInstrument {
    _private: (),
}

impl<T> BoundSyncInstrument<T> for NoopBoundSyncInstrument {
    fn measure(&self, _measurement: T) {
        // Ignored
    }
}
