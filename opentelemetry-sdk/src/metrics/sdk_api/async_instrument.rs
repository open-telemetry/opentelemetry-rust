//! Async metrics
use crate::{
    global,
    metrics::{sdk_api, MetricsError, Number},
    KeyValue,
};
use std::fmt;
use std::marker;
use std::sync::Arc;

/// Observation is used for reporting an asynchronous batch of metric values.
/// Instances of this type should be created by asynchronous instruments (e.g.,
/// [ValueObserver::observation]).
///
/// [ValueObserver::observation]: crate::metrics::ValueObserver::observation()
#[derive(Debug)]
pub struct Observation {
    number: Number,
    instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
}

impl Observation {
    /// Create a new observation for an instrument
    pub(crate) fn new(number: Number, instrument: Arc<dyn sdk_api::AsyncInstrumentCore>) -> Self {
        Observation { number, instrument }
    }

    /// The value of this observation
    pub fn number(&self) -> &Number {
        &self.number
    }
    /// The instrument used to record this observation
    pub fn instrument(&self) -> &Arc<dyn sdk_api::AsyncInstrumentCore> {
        &self.instrument
    }
}

/// A type of callback that `f64` observers run.
type F64ObserverCallback = Box<dyn Fn(ObserverResult<f64>) + Send + Sync>;

/// A type of callback that `u64` observers run.
type U64ObserverCallback = Box<dyn Fn(ObserverResult<u64>) + Send + Sync>;

/// A type of callback that `u64` observers run.
type I64ObserverCallback = Box<dyn Fn(ObserverResult<i64>) + Send + Sync>;

/// A callback argument for use with any Observer instrument that will be
/// reported as a batch of observations.
type BatchObserverCallback = Box<dyn Fn(BatchObserverResult) + Send + Sync>;

/// Data passed to an observer callback to capture observations for one
/// asynchronous metric instrument.
pub struct ObserverResult<T> {
    instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
    f: fn(&[KeyValue], &[Observation]),
    _marker: marker::PhantomData<T>,
}

impl<T> ObserverResult<T>
where
    T: Into<Number>,
{
    /// New observer result for a given metric instrument
    fn new(
        instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
        f: fn(&[KeyValue], &[Observation]),
    ) -> Self {
        ObserverResult {
            instrument,
            f,
            _marker: marker::PhantomData,
        }
    }

    /// Observe captures a single value from the associated instrument callback,
    /// with the given attributes.
    pub fn observe(&self, value: T, attributes: &[KeyValue]) {
        (self.f)(
            attributes,
            &[Observation {
                number: value.into(),
                instrument: self.instrument.clone(),
            }],
        )
    }
}

impl<T> fmt::Debug for ObserverResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObserverResult")
            .field("instrument", &self.instrument)
            .field("f", &"fn(&[KeyValue], &[Observation])")
            .finish()
    }
}

/// Passed to a batch observer callback to capture observations for multiple
/// asynchronous instruments.
pub struct BatchObserverResult {
    f: fn(&[KeyValue], &[Observation]),
}

impl fmt::Debug for BatchObserverResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BatchObserverResult")
            .field("f", &"fn(&[KeyValue], &[Observation])")
            .finish()
    }
}

impl BatchObserverResult {
    /// New observer result for a given metric instrument
    fn new(f: fn(&[KeyValue], &[Observation])) -> Self {
        BatchObserverResult { f }
    }

    /// Captures multiple observations from the associated batch instrument
    /// callback, with the given attributes.
    pub fn observe(&self, attributes: &[KeyValue], observations: &[Observation]) {
        (self.f)(attributes, observations)
    }
}
