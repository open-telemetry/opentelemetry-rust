//! # OpenTelemetry Metrics API

use std::result;
use std::sync::PoisonError;
use std::{borrow::Cow, sync::Arc};
use thiserror::Error;

mod instruments;
mod meter;
pub mod noop;

use crate::{Context, ExportError};
pub use instruments::{
    counter::{AsyncCounter, Counter, ObservableCounter, SyncCounter},
    gauge::{AsyncGauge, ObservableGauge},
    histogram::{Histogram, SyncHistogram},
    up_down_counter::{
        AsyncUpDownCounter, ObservableUpDownCounter, SyncUpDownCounter, UpDownCounter,
    },
    InstrumentBuilder,
};
pub use meter::{Meter, MeterProvider};

/// A specialized `Result` type for metric operations.
pub type Result<T> = result::Result<T, MetricsError>;

/// Errors returned by the metrics API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MetricsError {
    /// Other errors not covered by specific cases.
    #[error("Metrics error: {0}")]
    Other(String),
    /// Errors when requesting quantiles out of the 0-1 range.
    #[error("The requested quantile is out of range")]
    InvalidQuantile,
    /// Errors when recording nan values.
    #[error("NaN value is an invalid input")]
    NaNInput,
    /// Errors when recording negative values in monotonic sums.
    #[error("Negative value is out of range for this instrument")]
    NegativeInput,
    /// Errors when merging aggregators of incompatible types.
    #[error("Inconsistent aggregator types: {0}")]
    InconsistentAggregator(String),
    /// Errors when requesting data when no data has been collected
    #[error("No data collected by this aggregator")]
    NoDataCollected,
    /// Errors when registering to instruments with the same name and kind
    #[error("A metric was already registered by this name with another kind or number type: {0}")]
    MetricKindMismatch(String),
    /// Errors when processor logic is incorrect
    #[error("Inconsistent processor state")]
    InconsistentState,
    /// Errors when aggregator cannot subtract
    #[error("Aggregator does not subtract")]
    NoSubtraction,
    /// Fail to export metrics
    #[error("Metrics exporter {} failed with {0}", .0.exporter_name())]
    ExportErr(Box<dyn ExportError>),
}

impl<T: ExportError> From<T> for MetricsError {
    fn from(err: T) -> Self {
        MetricsError::ExportErr(Box::new(err))
    }
}

impl<T> From<PoisonError<T>> for MetricsError {
    fn from(err: PoisonError<T>) -> Self {
        MetricsError::Other(err.to_string())
    }
}

/// Units denote underlying data units tracked by `Meter`s.
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Unit(Cow<'static, str>);

impl Unit {
    /// Create a new `Unit` from an `Into<String>`
    pub fn new<S>(value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Unit(value.into())
    }

    /// View unit as &str
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Unit {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// SDK implemented trait for creating instruments
pub trait InstrumentProvider {
    /// creates an instrument for recording increasing values.
    fn u64_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<Counter<u64>> {
        Ok(Counter::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording increasing values.
    fn f64_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<Counter<f64>> {
        Ok(Counter::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording increasing values via callback.
    fn u64_observable_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableCounter<u64>> {
        Ok(ObservableCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording increasing values via callback.
    fn f64_observable_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableCounter<f64>> {
        Ok(ObservableCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording changes of a value.
    fn i64_up_down_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<UpDownCounter<i64>> {
        Ok(UpDownCounter::new(
            Arc::new(noop::NoopSyncInstrument::new()),
        ))
    }

    /// creates an instrument for recording changes of a value.
    fn f64_up_down_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<UpDownCounter<f64>> {
        Ok(UpDownCounter::new(
            Arc::new(noop::NoopSyncInstrument::new()),
        ))
    }

    /// creates an instrument for recording changes of a value.
    fn i64_observable_up_down_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        Ok(ObservableUpDownCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording changes of a value via callback.
    fn f64_observable_up_down_counter(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        Ok(ObservableUpDownCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording the current value via callback.
    fn u64_observable_gauge(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableGauge<u64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording the current value via callback.
    fn i64_observable_gauge(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableGauge<i64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording the current value via callback.
    fn f64_observable_gauge(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<ObservableGauge<f64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording a distribution of values.
    fn f64_histogram(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<Histogram<f64>> {
        Ok(Histogram::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording a distribution of values.
    fn u64_histogram(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<Histogram<u64>> {
        Ok(Histogram::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording a distribution of values.
    fn i64_histogram(
        &self,
        _name: String,
        _description: Option<String>,
        _unit: Option<Unit>,
    ) -> Result<Histogram<i64>> {
        Ok(Histogram::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// Captures the function that will be called during data collection.
    ///
    /// It is only valid to call `observe` within the scope of the passed function.
    fn register_callback(&self, callback: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()>;
}
