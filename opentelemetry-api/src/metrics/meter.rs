use core::fmt;
use std::sync::Arc;

use crate::metrics::{
    Counter, Histogram, InstrumentBuilder, InstrumentProvider, MetricsError, ObservableCounter,
    ObservableGauge, ObservableUpDownCounter, UpDownCounter,
};
use crate::{Context, InstrumentationLibrary};

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
    pub fn u64_counter(&self, name: impl Into<String>) -> InstrumentBuilder<'_, Counter<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values.
    pub fn f64_counter(&self, name: impl Into<String>) -> InstrumentBuilder<'_, Counter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values via callback.
    pub fn u64_observable_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableCounter<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording increasing values via callback.
    pub fn f64_observable_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableCounter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value.
    pub fn i64_up_down_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, UpDownCounter<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value.
    pub fn f64_up_down_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, UpDownCounter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value via callback.
    pub fn i64_observable_up_down_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableUpDownCounter<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording changes of a value via callback.
    pub fn f64_observable_up_down_counter(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableUpDownCounter<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn u64_observable_gauge(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableGauge<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn i64_observable_gauge(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableGauge<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording the current value via callback.
    pub fn f64_observable_gauge(
        &self,
        name: impl Into<String>,
    ) -> InstrumentBuilder<'_, ObservableGauge<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn f64_histogram(&self, name: impl Into<String>) -> InstrumentBuilder<'_, Histogram<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn u64_histogram(&self, name: impl Into<String>) -> InstrumentBuilder<'_, Histogram<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// creates an instrument builder for recording a distribution of values.
    pub fn i64_histogram(&self, name: impl Into<String>) -> InstrumentBuilder<'_, Histogram<i64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// Captures the function that will be called during data collection.
    ///
    /// It is only valid to call `observe` within the scope of the passed function.
    pub fn register_callback<F>(&self, callback: F) -> Result<(), MetricsError>
    where
        F: Fn(&Context) + Send + Sync + 'static,
    {
        self.instrument_provider
            .register_callback(Box::new(callback))
    }
}

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter")
            .field("instrumentation_library", &self.instrumentation_library)
            .finish()
    }
}
