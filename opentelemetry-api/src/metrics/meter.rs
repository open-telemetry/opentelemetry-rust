use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;

use crate::metrics::{
    AsyncInstrumentBuilder, Counter, Histogram, InstrumentBuilder, InstrumentProvider,
    ObservableCounter, ObservableGauge, ObservableUpDownCounter, Result, UpDownCounter,
};
use crate::{Context, InstrumentationLibrary, KeyValue};

use super::AsyncInstrument;

/// Returns named meter instances
pub trait MeterProvider {
    /// Creates a named [`Meter`] instance.
    fn meter(&self, name: &'static str) -> Meter {
        self.versioned_meter(name, None, None)
    }

    /// Creates an implementation of the [`Meter`] interface.
    ///
    /// The instrumentation name must be the name of the library providing instrumentation. This
    /// name may be the same as the instrumented code only if that code provides built-in
    /// instrumentation. If the instrumentation name is empty, then a implementation defined
    /// default name will be used instead.
    fn versioned_meter(
        &self,
        name: &'static str,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> Meter;
}

/// Provides access to instrument instances for recording metrics.
#[derive(Clone)]
pub struct Meter {
    pub(crate) instrumentation_library: InstrumentationLibrary,
    pub(crate) instrument_provider: Arc<dyn InstrumentProvider + Send + Sync>,
}

impl Meter {
    /// Create a new named meter from an instrumentation provider
    #[doc(hidden)]
    pub fn new(
        instrumentation_library: InstrumentationLibrary,
        instrument_provider: Arc<dyn InstrumentProvider + Send + Sync>,
    ) -> Self {
        Meter {
            instrumentation_library,
            instrument_provider,
        }
    }

    /// creates an instrument builder for recording increasing values.
    pub fn u64_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Counter<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values.
    pub fn f64_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Counter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values via callback.
    pub fn u64_observable_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values via callback.
    pub fn f64_observable_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value.
    pub fn i64_up_down_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, UpDownCounter<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value.
    pub fn f64_up_down_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, UpDownCounter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value via callback.
    pub fn i64_observable_up_down_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value via callback.
    pub fn f64_observable_up_down_counter(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn u64_observable_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn i64_observable_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn f64_observable_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64> {
        AsyncInstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn f64_histogram(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Histogram<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn u64_histogram(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Histogram<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn i64_histogram(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Histogram<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// Registers a callback to be called during the collection of a measurement
    /// cycle.
    ///
    /// The instruments passed as arguments to be registered are the only
    /// instruments that may observe values.
    ///
    /// If no instruments are passed, the callback will not be registered.
    pub fn register_callback<F>(
        &self,
        instruments: &[Arc<dyn Any>],
        callback: F,
    ) -> Result<Box<dyn Registration>>
    where
        F: Fn(&Context, &dyn Observer) + Send + Sync + 'static,
    {
        self.instrument_provider
            .register_callback(instruments, Box::new(callback))
    }
}

/// A token representing the unique registration of a callback for a set of
/// instruments with a [Meter].
pub trait Registration {
    /// Removes the callback registration from its associated [Meter].
    ///
    /// This method needs to be idempotent and concurrent safe.
    fn unregister(&mut self) -> Result<()>;
}

/// Records measurements for multiple instruments in a callback.
pub trait Observer {
    /// Records the f64 value with attributes for the observable.
    fn observe_f64(
        &self,
        cx: &Context,
        inst: &dyn AsyncInstrument<f64>,
        measurement: f64,
        attrs: &[KeyValue],
    );

    /// Records the u64 value with attributes for the observable.
    fn observe_u64(
        &self,
        cx: &Context,
        inst: &dyn AsyncInstrument<u64>,
        measurement: u64,
        attrs: &[KeyValue],
    );

    /// Records the i64 value with attributes for the observable.
    fn observe_i64(
        &self,
        cx: &Context,
        inst: &dyn AsyncInstrument<i64>,
        measurement: i64,
        attrs: &[KeyValue],
    );
}

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter")
            .field("instrumentation_library", &self.instrumentation_library)
            .finish()
    }
}
