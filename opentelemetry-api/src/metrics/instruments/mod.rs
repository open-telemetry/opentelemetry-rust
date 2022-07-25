use crate::metrics::{Meter, MetricsError, Result, Unit};
use core::fmt;
use std::convert::TryFrom;
use std::marker;

pub(super) mod counter;
pub(super) mod gauge;
pub(super) mod histogram;
pub(super) mod up_down_counter;

/// Configuration for building an instrument.
pub struct InstrumentBuilder<'a, T> {
    meter: &'a Meter,
    name: String,
    description: Option<String>,
    unit: Option<Unit>,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> InstrumentBuilder<'a, T>
where
    T: TryFrom<Self, Error = MetricsError>,
{
    /// Create a new counter builder
    pub(crate) fn new(meter: &'a Meter, name: String) -> Self {
        InstrumentBuilder {
            meter,
            name,
            description: None,
            unit: None,
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this counter
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the unit for this counter.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Creates a new counter instrument.
    pub fn try_init(self) -> Result<T> {
        T::try_from(self)
    }

    /// Creates a new counter instrument.
    ///
    /// # Panics
    ///
    /// This function panics if the instrument cannot be created. Use try_init if you want to
    /// handle errors.
    pub fn init(self) -> T {
        self.try_init().unwrap()
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
