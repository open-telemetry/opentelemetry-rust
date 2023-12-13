use crate::attributes::AttributeSet;
use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;

#[cfg(feature = "otel_unstable")]
use crate::metrics::Gauge;
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
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, metrics::MeterProvider};
    /// use opentelemetry::KeyValue;
    ///
    /// let provider = global::meter_provider();
    ///
    /// // meter used in applications
    /// let meter = provider.meter("my_app");
    ///
    /// // meter used in libraries/crates that optionally includes version and schema url
    /// let meter = provider.versioned_meter(
    ///     "my_library",
    ///     Some(env!("CARGO_PKG_VERSION")),
    ///     Some("https://opentelemetry.io/schema/1.0.0"),
    ///     Some(vec![KeyValue::new("key", "value")]),
    /// );
    /// ```
    fn meter(&self, name: impl Into<Cow<'static, str>>) -> Meter {
        self.versioned_meter(
            name,
            None::<Cow<'static, str>>,
            None::<Cow<'static, str>>,
            None,
        )
    }

    /// Returns a new versioned meter with a given name.
    ///
    /// The instrumentation name must be the name of the library providing instrumentation. This
    /// name may be the same as the instrumented code only if that code provides built-in
    /// instrumentation. If the instrumentation name is empty, then a implementation defined
    /// default name will be used instead.
    fn versioned_meter(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Meter;
}

/// Provides access to instrument instances for recording measurements.
///
/// ```
/// use opentelemetry::{attributes::AttributeSet, global, KeyValue};
///
/// let meter = global::meter("my-meter");
///
/// // Meters can create metric instruments that can record values of type u64 and f64
///
/// // u64 Counter
/// let u64_counter = meter.u64_counter("my_u64_counter").init();
///
/// // Define the attributes the counters will use
/// let attributes = AttributeSet::from([
///         KeyValue::new("mykey1", "myvalue1"),
///         KeyValue::new("mykey2", "myvalue2"),
///     ]);
///
/// // Record measurements using the counter instrument add()
/// u64_counter.add(10, attributes.clone());
///
/// // f64 Counter
/// let f64_counter = meter.f64_counter("my_f64_counter").init();
///
/// // Record measurements using the counter instrument add()
/// f64_counter.add(3.15, attributes.clone());
///
/// // u6 observable counter
/// let observable_u4_counter = meter.u64_observable_counter("my_observable_u64_counter").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[observable_u4_counter.as_any()], move |observer| {
///     observer.observe_u64(&observable_u4_counter, 1, observer_attributes.clone())
/// });
///
/// // f64 observable counter
/// let observable_f64_counter = meter.f64_observable_counter("my_observable_f64_counter").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[observable_f64_counter.as_any()], move |observer| {
///     observer.observe_f64(&observable_f64_counter, 1.55, observer_attributes.clone())
/// });
///
/// // i64 updown counter
/// let updown_i64_counter = meter.i64_up_down_counter("my_updown_i64_counter").init();
///
/// // Record measurements using the updown counter instrument add()
/// updown_i64_counter.add(-10, attributes.clone());
///
/// // f64 updown counter
/// let updown_f64_counter = meter.f64_up_down_counter("my_updown_f64_counter").init();
///
/// // Record measurements using the updown counter instrument add()
/// updown_f64_counter.add(-10.67, attributes.clone());
///
/// // i64 observable updown counter
/// let observable_i64_up_down_counter = meter.i64_observable_up_down_counter("my_observable_i64_updown_counter").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[observable_i64_up_down_counter.as_any()], move |observer| {
///     observer.observe_i64(&observable_i64_up_down_counter, 1, observer_attributes.clone())
/// });
///
/// // f64 observable updown counter
/// let observable_f64_up_down_counter = meter.f64_observable_up_down_counter("my_observable_f64_updown_counter").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[observable_f64_up_down_counter.as_any()], move |observer| {
///     observer.observe_f64(&observable_f64_up_down_counter, 1.16, observer_attributes.clone())
/// });
///
/// // Observable f64 gauge
/// let f64_gauge = meter.f64_observable_gauge("my_f64_gauge").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[f64_gauge.as_any()], move |observer| {
///     observer.observe_f64(&f64_gauge, 2.32, observer_attributes.clone())
/// });
///
/// // Observable i64 gauge
/// let i64_gauge = meter.i64_observable_gauge("my_i64_gauge").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[i64_gauge.as_any()], move |observer| {
///     observer.observe_i64(&i64_gauge, 12, observer_attributes.clone())
/// });
///
/// // Observable u64 gauge
/// let u64_gauge = meter.u64_observable_gauge("my_u64_gauge").init();
///
/// // Register a callback to this meter for an asynchronous instrument to record measurements
/// let observer_attributes = attributes.clone();
/// meter.register_callback(&[u64_gauge.as_any()], move |observer| {
///     observer.observe_u64(&u64_gauge, 1, observer_attributes.clone())
/// });
///
/// // f64 histogram
/// let f64_histogram = meter.f64_histogram("my_f64_histogram").init();
///
/// // Record measurements using the histogram instrument record()
/// f64_histogram.record(10.5, attributes.clone());
///
/// // u64 histogram
/// let u64_histogram = meter.u64_histogram("my_u64_histogram").init();
///
/// // Record measurements using the histogram instrument record()
/// u64_histogram.record(12, attributes);
///
/// ```
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

    /// # Experimental
    /// This method is experimental and can be changed/removed in future releases.
    /// creates an instrument builder for recording independent values.
    #[cfg(feature = "otel_unstable")]
    pub fn u64_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Gauge<u64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// # Experimental
    /// This method is experimental and can be changed/removed in future releases.
    /// creates an instrument builder for recording independent values.
    #[cfg(feature = "otel_unstable")]
    pub fn f64_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Gauge<f64>> {
        InstrumentBuilder::new(self, name.into())
    }

    /// # Experimental
    /// This method is experimental and can be changed/removed in future releases.
    /// creates an instrument builder for recording indenpendent values.
    #[cfg(feature = "otel_unstable")]
    pub fn i64_gauge(
        &self,
        name: impl Into<Cow<'static, str>>,
    ) -> InstrumentBuilder<'_, Gauge<i64>> {
        InstrumentBuilder::new(self, name.into())
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
    fn observe_f64(&self, inst: &dyn AsyncInstrument<f64>, measurement: f64, attrs: AttributeSet);

    /// Records the u64 value with attributes for the observable.
    fn observe_u64(&self, inst: &dyn AsyncInstrument<u64>, measurement: u64, attrs: AttributeSet);

    /// Records the i64 value with attributes for the observable.
    fn observe_i64(&self, inst: &dyn AsyncInstrument<i64>, measurement: i64, attrs: AttributeSet);
}

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Meter")
    }
}
