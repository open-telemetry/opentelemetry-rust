use crate::api::{
    metrics::{
        sdk_api, AsyncRunner, BatchObserver, BatchObserverCallback, CounterBuilder, Descriptor,
        Measurement, NumberKind, ObserverResult, Result, SumObserverBuilder, UpDownCounterBuilder,
        UpDownSumObserverBuilder, ValueObserverBuilder, ValueRecorderBuilder,
    },
    Context, KeyValue,
};
use std::fmt;
use std::sync::Arc;

/// Returns named meter instances
pub trait MeterProvider: fmt::Debug {
    /// Creates an implementation of the [`Meter`] interface. The
    /// instrumentation name must be the name of the library providing
    /// instrumentation. This name may be the same as the instrumented code only if
    /// that code provides built-in instrumentation. If the instrumentation name is
    /// empty, then a implementation defined default name will be used instead.
    ///
    /// [`Meter`]: struct.Meter.html
    fn meter(&self, instrumentation_name: &str) -> Meter;
}

/// Meter is the OpenTelemetry metric API, based on a sdk-defined `MeterCore`
/// implementation and the `Meter` library name.
#[derive(Debug)]
pub struct Meter {
    pub(crate) instrumentation_name: String,
    pub(crate) core: Arc<dyn sdk_api::MeterCore + Send + Sync>,
}

impl Meter {
    /// Create a new named meter from a sdk implemented core
    pub fn new<T: Into<String>>(
        instrumentation_name: T,
        core: Arc<dyn sdk_api::MeterCore + Send + Sync>,
    ) -> Self {
        Meter {
            instrumentation_name: instrumentation_name.into(),
            core,
        }
    }

    /// Creates a new floating point `ValueObserverBuilder` instrument with the
    /// given name and callback
    pub fn f64_value_observer<T, F>(&self, name: T, callback: F) -> ValueObserverBuilder<f64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<f64>) + Send + Sync + 'static,
    {
        ValueObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::F64(Box::new(callback)),
            NumberKind::F64,
        )
    }

    /// Creates a new integral `ValueObserverBuilder` instrument with the given name
    /// and callback
    pub fn u64_value_observer<T, F>(&self, name: T, callback: F) -> ValueObserverBuilder<u64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<u64>) + Send + Sync + 'static,
    {
        ValueObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::U64(Box::new(callback)),
            NumberKind::U64,
        )
    }

    /// Creates a new integral `ValueObserverBuilder` instrument with the given name
    /// and callback
    pub fn i64_value_observer<T, F>(&self, name: T, callback: F) -> ValueObserverBuilder<i64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<i64>) + Send + Sync + 'static,
    {
        ValueObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::I64(Box::new(callback)),
            NumberKind::I64,
        )
    }

    /// Creates a new floating point `SumObserverBuilder` instrument with the given
    /// name and callback
    pub fn f64_sum_observer<T, F>(&self, name: T, callback: F) -> SumObserverBuilder<f64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<f64>) + Send + Sync + 'static,
    {
        SumObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::F64(Box::new(callback)),
            NumberKind::F64,
        )
    }

    /// Creates a new integral `SumObserverBuilder` instrument with the given name
    /// and callback
    pub fn u64_sum_observer<T, F>(&self, name: T, callback: F) -> SumObserverBuilder<u64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<u64>) + Send + Sync + 'static,
    {
        SumObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::U64(Box::new(callback)),
            NumberKind::U64,
        )
    }

    /// Creates a new floating point `UpDownSumObserverBuilder` instrument with the
    /// given name and callback
    pub fn f64_up_down_sum_observer<T, F>(
        &self,
        name: T,
        callback: F,
    ) -> UpDownSumObserverBuilder<f64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<f64>) + Send + Sync + 'static,
    {
        UpDownSumObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::F64(Box::new(callback)),
            NumberKind::F64,
        )
    }

    /// Creates a new integral `SumUpDownObserverBuilder` instrument with the given
    /// name and callback
    pub fn i64_up_down_sum_observer<T, F>(
        &self,
        name: T,
        callback: F,
    ) -> UpDownSumObserverBuilder<i64>
    where
        T: Into<String>,
        F: Fn(ObserverResult<i64>) + Send + Sync + 'static,
    {
        UpDownSumObserverBuilder::new(
            self,
            name.into(),
            AsyncRunner::I64(Box::new(callback)),
            NumberKind::I64,
        )
    }

    /// Creates a new `ValueRecorderBuilder` for `f64` values with the given name.
    pub fn f64_value_recorder<T>(&self, name: T) -> ValueRecorderBuilder<f64>
    where
        T: Into<String>,
    {
        ValueRecorderBuilder::new(self, name.into(), NumberKind::F64)
    }

    /// Creates a new `ValueRecorderBuilder` for `i64` values with the given name.
    pub fn i64_value_recorder<T>(&self, name: T) -> ValueRecorderBuilder<i64>
    where
        T: Into<String>,
    {
        ValueRecorderBuilder::new(self, name.into(), NumberKind::I64)
    }

    /// Creates a new `ValueRecorderBuilder` for `u64` values with the given name.
    pub fn u64_value_recorder<T>(&self, name: T) -> ValueRecorderBuilder<u64>
    where
        T: Into<String>,
    {
        ValueRecorderBuilder::new(self, name.into(), NumberKind::U64)
    }

    /// Creates a new integer `CounterBuilder` for `u64` values with the given name.
    pub fn u64_counter<T>(&self, name: T) -> CounterBuilder<u64>
    where
        T: Into<String>,
    {
        CounterBuilder::new(self, name.into(), NumberKind::U64)
    }

    /// Creates a new floating point `CounterBuilder` for `f64` values with the given name.
    pub fn f64_counter<T>(&self, name: T) -> CounterBuilder<f64>
    where
        T: Into<String>,
    {
        CounterBuilder::new(self, name.into(), NumberKind::F64)
    }

    /// Creates a new integer `UpDownCounterBuilder` for an `i64` up down counter with the given name.
    pub fn i64_up_down_counter<T>(&self, name: T) -> UpDownCounterBuilder<i64>
    where
        T: Into<String>,
    {
        UpDownCounterBuilder::new(self, name.into(), NumberKind::I64)
    }

    /// Creates a new floating point `UpDownCounterBuilder` for an `f64` up down counter with the given name.
    pub fn f64_up_down_counter<T>(&self, name: T) -> UpDownCounterBuilder<i64>
    where
        T: Into<String>,
    {
        UpDownCounterBuilder::new(self, name.into(), NumberKind::F64)
    }

    /// Creates a new `BatchObserver` that supports making batches of observations for
    /// multiple instruments.
    pub fn batch_observer(&self, callback: BatchObserverCallback) -> BatchObserver {
        BatchObserver::new(self, AsyncRunner::Batch(callback))
    }

    /// Atomically record a batch of measurements.
    pub fn record_batch<T: IntoIterator<Item = Measurement>>(
        &self,
        labels: &[KeyValue],
        measurements: T,
    ) {
        self.record_batch_with_context(&Context::current(), labels, measurements)
    }

    /// Atomically record a batch of measurements with a given context
    pub fn record_batch_with_context<T: IntoIterator<Item = Measurement>>(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: T,
    ) {
        self.core
            .record_batch_with_context(cx, labels, measurements.into_iter().collect())
    }

    pub(crate) fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn sdk_api::SyncInstrumentCore + Send + Sync>> {
        self.core.new_sync_instrument(descriptor)
    }

    pub(crate) fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn sdk_api::AsyncInstrumentCore + Send + Sync>> {
        self.core.new_async_instrument(descriptor, runner)
    }
}
