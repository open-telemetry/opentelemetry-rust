use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;

use crate::metrics::{
    AsyncInstrumentBuilder, Counter, Histogram, InstrumentBuilder, InstrumentProvider,
    ObservableCounter, ObservableGauge, ObservableUpDownCounter, Result, UpDownCounter,
};
use crate::KeyValue;

use super::AsyncInstrument;

/// Provides access to named [Meter] instances, for instrumenting an application
/// or crate.
pub trait MeterProvider {
    /// Returns a new [Meter] with the provided name and default configuration.
    ///
    /// A [Meter] should be scoped at most to a single application or crate. The
    /// name needs to be unique so it does not collide with other names used by
    /// an application, nor other applications.
    ///
    /// If the name is empty, then an implementation defined default name will
    /// be used instead.
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
    pub(crate) instrument_provider: Arc<dyn InstrumentProvider + Send + Sync>,
}

impl Meter {
    /// Create a new named meter from an instrumentation provider
    #[doc(hidden)]
    pub fn new(instrument_provider: Arc<dyn InstrumentProvider + Send + Sync>) -> Self {
        Meter {
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
    ) -> Result<Box<dyn CallbackRegistration>>
    where
        F: Fn(&dyn Observer) + Send + Sync + 'static,
    {
        self.instrument_provider
            .register_callback(instruments, Box::new(callback))
    }
}

/// A token representing the unique registration of a callback for a set of
/// instruments with a [Meter].
pub trait CallbackRegistration: Send + Sync {
    /// Removes the callback registration from its associated [Meter].
    fn unregister(&mut self) -> Result<()>;
}

/// Records measurements for multiple instruments in a callback.
pub trait Observer {
    /// Records the f64 value with attributes for the observable.
    fn observe_f64(&self, inst: &dyn AsyncInstrument<f64>, measurement: f64, attrs: &[KeyValue]);

    /// Records the u64 value with attributes for the observable.
    fn observe_u64(&self, inst: &dyn AsyncInstrument<u64>, measurement: u64, attrs: &[KeyValue]);

    /// Records the i64 value with attributes for the observable.
    fn observe_i64(&self, inst: &dyn AsyncInstrument<i64>, measurement: i64, attrs: &[KeyValue]);
}

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Meter")
    }
}
