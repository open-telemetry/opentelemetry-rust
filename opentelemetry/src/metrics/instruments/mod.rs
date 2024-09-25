use crate::metrics::{Meter, MetricsError, Result};
use crate::KeyValue;
use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use std::marker;
use std::sync::Arc;

use super::{Histogram, InstrumentProvider};

pub(super) mod counter;
pub(super) mod gauge;
pub(super) mod histogram;
pub(super) mod up_down_counter;

/// An SDK implemented instrument that records measurements via callback.
pub trait AsyncInstrument<T>: Send + Sync {
    /// Observes the state of the instrument.
    ///
    /// It is only valid to call this within a callback.
    fn observe(&self, measurement: T, attributes: &[KeyValue]);

    /// Used for SDKs to downcast instruments in callbacks.
    fn as_any(&self) -> Arc<dyn Any>;
}

/// Configuration for building a Histogram.
#[non_exhaustive] // We expect to add more configuration fields in the future
pub struct HistogramBuilder<'a, T> {
    /// Instrument provider is used to create the instrument.
    pub instrument_provider: &'a dyn InstrumentProvider,

    /// Name of the Histogram.
    pub name: Cow<'static, str>,

    /// Description of the Histogram.
    pub description: Option<Cow<'static, str>>,

    /// Unit of the Histogram.
    pub unit: Option<Cow<'static, str>>,

    /// Bucket boundaries for the histogram.
    pub boundaries: Option<Vec<f64>>,

    // boundaries: Vec<T>,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> HistogramBuilder<'a, T> {
    /// Create a new instrument builder
    pub(crate) fn new(meter: &'a Meter, name: Cow<'static, str>) -> Self {
        HistogramBuilder {
            instrument_provider: meter.instrument_provider.as_ref(),
            name,
            description: None,
            unit: None,
            boundaries: None,
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this instrument
    pub fn with_description<S: Into<Cow<'static, str>>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the unit for this instrument.
    ///
    /// Unit is case sensitive(`kb` is not the same as `kB`).
    ///
    /// Unit must be:
    /// - ASCII string
    /// - No longer than 63 characters
    pub fn with_unit<S: Into<Cow<'static, str>>>(mut self, unit: S) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the boundaries for this histogram.
    ///
    /// Setting boundaries is optional. By default, the boundaries are set to:
    ///
    /// `[0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0, 10000.0]`
    pub fn with_boundaries(mut self, boundaries: Vec<f64>) -> Self {
        self.boundaries = Some(boundaries);
        self
    }
}

impl<'a> HistogramBuilder<'a, f64> {
    /// Validate the instrument configuration and creates a new instrument.
    pub fn try_init(self) -> Result<Histogram<f64>> {
        self.instrument_provider.f64_histogram(self)
    }

    /// Creates a new instrument.
    ///
    /// Validate the instrument configuration and crates a new instrument.
    ///
    /// # Panics
    ///
    /// Panics if the instrument cannot be created. Use
    /// [`try_init`](InstrumentBuilder::try_init) if you want to handle errors.
    pub fn init(self) -> Histogram<f64> {
        self.try_init().unwrap()
    }
}

impl<'a> HistogramBuilder<'a, u64> {
    /// Validate the instrument configuration and creates a new instrument.
    pub fn try_init(self) -> Result<Histogram<u64>> {
        self.instrument_provider.u64_histogram(self)
    }

    /// Creates a new instrument.
    ///
    /// Validate the instrument configuration and crates a new instrument.
    ///
    /// # Panics
    ///
    /// Panics if the instrument cannot be created. Use
    /// [`try_init`](InstrumentBuilder::try_init) if you want to handle errors.
    pub fn init(self) -> Histogram<u64> {
        self.try_init().unwrap()
    }
}

/// Configuration for building a sync instrument.
#[non_exhaustive] // We expect to add more configuration fields in the future
pub struct InstrumentBuilder<'a, T> {
    /// Instrument provider is used to create the instrument.
    pub instrument_provider: &'a dyn InstrumentProvider,

    /// Name of the instrument.
    pub name: Cow<'static, str>,

    /// Description of the instrument.
    pub description: Option<Cow<'static, str>>,

    /// Unit of the instrument.
    pub unit: Option<Cow<'static, str>>,

    _marker: marker::PhantomData<T>,
}

impl<'a, T> InstrumentBuilder<'a, T>
where
    T: TryFrom<Self, Error = MetricsError>,
{
    /// Create a new instrument builder
    pub(crate) fn new(meter: &'a Meter, name: Cow<'static, str>) -> Self {
        InstrumentBuilder {
            instrument_provider: meter.instrument_provider.as_ref(),
            name,
            description: None,
            unit: None,
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this instrument
    pub fn with_description<S: Into<Cow<'static, str>>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the unit for this instrument.
    ///
    /// Unit is case sensitive(`kb` is not the same as `kB`).
    ///
    /// Unit must be:
    /// - ASCII string
    /// - No longer than 63 characters
    pub fn with_unit<S: Into<Cow<'static, str>>>(mut self, unit: S) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Validate the instrument configuration and creates a new instrument.
    pub fn try_init(self) -> Result<T> {
        T::try_from(self)
    }

    /// Creates a new instrument.
    ///
    /// Validate the instrument configuration and crates a new instrument.
    ///
    /// # Panics
    ///
    /// Panics if the instrument cannot be created. Use
    /// [`try_init`](InstrumentBuilder::try_init) if you want to handle errors.
    pub fn init(self) -> T {
        T::try_from(self).unwrap()
    }
}

impl<T> fmt::Debug for InstrumentBuilder<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentBuilder")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("kind", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T> fmt::Debug for HistogramBuilder<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistogramBuilder")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("boundaries", &self.boundaries)
            .field(
                "kind",
                &format!("Histogram<{}>", &std::any::type_name::<T>()),
            )
            .finish()
    }
}

/// A function registered with a [Meter] that makes observations for the
/// instruments it is registered with.
///
/// The async instrument parameter is used to record measurement observations
/// for these instruments.
///
/// The function needs to complete in a finite amount of time.
pub type Callback<T> = Box<dyn Fn(&dyn AsyncInstrument<T>) + Send + Sync>;

/// Configuration for building an async instrument.
#[non_exhaustive] // We expect to add more configuration fields in the future
pub struct AsyncInstrumentBuilder<'a, I, M>
where
    I: AsyncInstrument<M>,
{
    /// Instrument provider is used to create the instrument.
    pub instrument_provider: &'a dyn InstrumentProvider,

    /// Name of the instrument.
    pub name: Cow<'static, str>,

    /// Description of the instrument.
    pub description: Option<Cow<'static, str>>,

    /// Unit of the instrument.
    pub unit: Option<Cow<'static, str>>,

    /// Callbacks to be called for this instrument.
    pub callbacks: Vec<Callback<M>>,

    _inst: marker::PhantomData<I>,
}

impl<'a, I, M> AsyncInstrumentBuilder<'a, I, M>
where
    I: TryFrom<Self, Error = MetricsError>,
    I: AsyncInstrument<M>,
{
    /// Create a new instrument builder
    pub(crate) fn new(meter: &'a Meter, name: Cow<'static, str>) -> Self {
        AsyncInstrumentBuilder {
            instrument_provider: meter.instrument_provider.as_ref(),
            name,
            description: None,
            unit: None,
            _inst: marker::PhantomData,
            callbacks: Vec::new(),
        }
    }

    /// Set the description for this instrument
    pub fn with_description<S: Into<Cow<'static, str>>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the unit for this instrument.
    ///
    /// Unit is case sensitive(`kb` is not the same as `kB`).
    ///
    /// Unit must be:
    /// - ASCII string
    /// - No longer than 63 characters
    pub fn with_unit<S: Into<Cow<'static, str>>>(mut self, unit: S) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the callback to be called for this instrument.
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&dyn AsyncInstrument<M>) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
        self
    }

    /// Validate the instrument configuration and creates a new instrument.
    pub fn try_init(self) -> Result<I> {
        I::try_from(self)
    }

    /// Creates a new instrument.
    ///
    /// Validate the instrument configuration and creates a new instrument.
    ///
    /// # Panics
    ///
    /// Panics if the instrument cannot be created. Use
    /// [`try_init`](InstrumentBuilder::try_init) if you want to handle errors.
    pub fn init(self) -> I {
        I::try_from(self).unwrap()
    }
}

impl<I, M> fmt::Debug for AsyncInstrumentBuilder<'_, I, M>
where
    I: AsyncInstrument<M>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentBuilder")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("kind", &std::any::type_name::<I>())
            .field("callbacks_len", &self.callbacks.len())
            .finish()
    }
}
