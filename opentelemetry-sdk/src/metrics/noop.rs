use opentelemetry::{
    metrics::{AsyncInstrument, InstrumentProvider, SyncInstrument},
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
}

/// A no-op async instrument.
#[derive(Debug, Default)]
pub(crate) struct NoopAsyncInstrument {
    _private: (),
}

impl NoopAsyncInstrument {
    /// Create a new no-op async instrument
    pub(crate) fn new() -> Self {
        NoopAsyncInstrument { _private: () }
    }
}

impl<T> AsyncInstrument<T> for NoopAsyncInstrument {
    fn observe(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}
