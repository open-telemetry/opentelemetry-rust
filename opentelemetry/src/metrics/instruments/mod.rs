use crate::metrics::{Meter, MetricsError, Result, Unit};
use crate::KeyValue;
use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use std::marker;
use std::sync::Arc;

use super::InstrumentProvider;

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

/// Configuration for building a sync instrument.
pub struct InstrumentBuilder<T> {
    instrument_provider: Arc<dyn InstrumentProvider + Send + Sync>,
    name: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    unit: Option<Unit>,
    _marker: marker::PhantomData<T>,
}

impl<T> InstrumentBuilder<T>
where
    T: TryFrom<Self, Error = MetricsError>,
{
    /// Create a new instrument builder
    pub(crate) fn new(meter: &Meter, name: Cow<'static, str>) -> Self {
        InstrumentBuilder {
            instrument_provider: meter.instrument_provider.clone(),
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
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
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

impl<T> fmt::Debug for InstrumentBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentBuilder")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("kind", &std::any::type_name::<T>())
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
pub struct AsyncInstrumentBuilder<'a, I, M>
where
    I: AsyncInstrument<M>,
{
    meter: &'a Meter,
    name: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    unit: Option<Unit>,
    _inst: marker::PhantomData<I>,
    callbacks: Vec<Callback<M>>,
}

impl<'a, I, M> AsyncInstrumentBuilder<'a, I, M>
where
    I: TryFrom<Self, Error = MetricsError>,
    I: AsyncInstrument<M>,
{
    /// Create a new instrument builder
    pub(crate) fn new(meter: &'a Meter, name: Cow<'static, str>) -> Self {
        AsyncInstrumentBuilder {
            meter,
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
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
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
