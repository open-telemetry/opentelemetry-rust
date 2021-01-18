use crate::metrics::{
    sdk_api, AsyncRunner, Descriptor, InstrumentKind, Meter, Number, NumberKind, Observation,
    Result,
};
use crate::Unit;
use std::sync::Arc;

/// An Observer callback that can report observations for multiple instruments.
#[derive(Debug)]
pub struct BatchObserver<'a> {
    meter: &'a Meter,
    runner: AsyncRunner,
}

impl<'a> BatchObserver<'a> {
    pub(crate) fn new(meter: &'a Meter, runner: AsyncRunner) -> Self {
        BatchObserver { meter, runner }
    }
}

/// A metric that captures a precomputed sum of values at a point in time.
#[derive(Debug)]
pub struct SumObserver<T> {
    instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> SumObserver<T>
where
    T: Into<Number>,
{
    /// Returns an `Observation`: a `BatchObserverCallback` argument, for an
    /// asynchronous instrument. This returns an implementation-level
    /// object for use by the SDK, users should not refer to this.
    pub fn observation(&self, value: T) -> Observation {
        Observation::new(value.into(), self.instrument.clone())
    }
}

/// Configuration options for building a `SumObserver`
#[derive(Debug)]
pub struct SumObserverBuilder<'a, T> {
    meter: &'a Meter,
    descriptor: Descriptor,
    runner: AsyncRunner,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> SumObserverBuilder<'a, T> {
    pub(crate) fn new(
        meter: &'a Meter,
        name: String,
        runner: AsyncRunner,
        number_kind: NumberKind,
    ) -> Self {
        SumObserverBuilder {
            meter,
            descriptor: Descriptor::new(
                name,
                meter.instrumentation_library().name,
                meter.instrumentation_library().version,
                InstrumentKind::SumObserver,
                number_kind,
            ),
            runner,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the description of this `SumObserver`
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// Set the unit for this `SumObserver`.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.descriptor.set_unit(unit);
        self
    }

    /// Create a `SumObserver` from this configuration.
    pub fn try_init(self) -> Result<SumObserver<T>> {
        let instrument = self
            .meter
            .new_async_instrument(self.descriptor, self.runner)?;

        Ok(SumObserver {
            instrument,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a `SumObserver` from this configuration.
    ///
    /// # Panics
    ///
    /// This method panics if it cannot create an instrument with the provided
    /// config. If you want to handle results instead, use [`try_init`]
    ///
    /// [`try_init`]: struct.SumObserverBuilder.html#method.try_init
    pub fn init(self) -> SumObserver<T> {
        SumObserver {
            instrument: self
                .meter
                .new_async_instrument(self.descriptor, self.runner)
                .unwrap(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// A metric that captures a precomputed non-monotonic sum of values at a point
/// in time.
#[derive(Debug)]
pub struct UpDownSumObserver<T> {
    instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> UpDownSumObserver<T>
where
    T: Into<Number>,
{
    /// Returns an `Observation`: a `BatchObserverCallback` argument, for an
    /// asynchronous instrument. This returns an implementation-level
    /// object for use by the SDK, users should not refer to this.
    pub fn observation(&self, value: T) -> Observation {
        Observation::new(value.into(), self.instrument.clone())
    }
}

/// Configuration options for building a `UpDownSumObserver`
#[derive(Debug)]
pub struct UpDownSumObserverBuilder<'a, T> {
    meter: &'a Meter,
    descriptor: Descriptor,
    runner: AsyncRunner,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> UpDownSumObserverBuilder<'a, T> {
    pub(crate) fn new(
        meter: &'a Meter,
        name: String,
        runner: AsyncRunner,
        number_kind: NumberKind,
    ) -> Self {
        UpDownSumObserverBuilder {
            meter,
            descriptor: Descriptor::new(
                name,
                meter.instrumentation_library().name,
                meter.instrumentation_library().version,
                InstrumentKind::UpDownSumObserver,
                number_kind,
            ),
            runner,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the description of this `UpDownSumObserver`
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// Set the unit for this `UpDownSumObserver`.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.descriptor.set_unit(unit);
        self
    }

    /// Create a `UpDownSumObserver` from this configuration.
    pub fn try_init(self) -> Result<UpDownSumObserver<T>> {
        let instrument = self
            .meter
            .new_async_instrument(self.descriptor, self.runner)?;

        Ok(UpDownSumObserver {
            instrument,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a `UpDownSumObserver` from this configuration.
    ///
    /// # Panics
    ///
    /// This method panics if it cannot create an instrument with the provided
    /// config. If you want to handle results instead, use [`try_init`]
    ///
    /// [`try_init`]: struct.UpDownSumObserverBuilder.html#method.try_init
    pub fn init(self) -> UpDownSumObserver<T> {
        UpDownSumObserver {
            instrument: self
                .meter
                .new_async_instrument(self.descriptor, self.runner)
                .unwrap(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// A metric that captures a set of values at a point in time.
#[derive(Debug)]
pub struct ValueObserver<T> {
    instrument: Arc<dyn sdk_api::AsyncInstrumentCore>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ValueObserver<T>
where
    T: Into<Number>,
{
    /// Returns an `Observation`: a `BatchObserverCallback` argument, for an
    /// asynchronous instrument. This returns an implementation-level
    /// object for use by the SDK, users should not refer to this.
    pub fn observation(&self, value: T) -> Observation {
        Observation::new(value.into(), self.instrument.clone())
    }
}

/// Configuration options for building a `ValueObserver`
#[derive(Debug)]
pub struct ValueObserverBuilder<'a, T> {
    meter: &'a Meter,
    descriptor: Descriptor,
    runner: AsyncRunner,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> ValueObserverBuilder<'a, T> {
    pub(crate) fn new(
        meter: &'a Meter,
        name: String,
        runner: AsyncRunner,
        number_kind: NumberKind,
    ) -> Self {
        ValueObserverBuilder {
            meter,
            descriptor: Descriptor::new(
                name,
                meter.instrumentation_library().name,
                meter.instrumentation_library().version,
                InstrumentKind::ValueObserver,
                number_kind,
            ),
            runner,
            _marker: std::marker::PhantomData,
        }
    }
    /// Set the description of this `ValueObserver`
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// Set the unit for this `ValueObserver`.
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.descriptor.set_unit(unit);
        self
    }

    /// Create a `ValueObserver` from this configuration.
    pub fn try_init(self) -> Result<ValueObserver<T>> {
        let instrument = self
            .meter
            .new_async_instrument(self.descriptor, self.runner)?;

        Ok(ValueObserver {
            instrument,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a `ValueObserver` from this configuration.
    ///
    /// # Panics
    ///
    /// This method panics if it cannot create an instrument with the provided
    /// config. If you want to handle results instead, use [`try_init`]
    ///
    /// [`try_init`]: struct.ValueObserverBuilder.html#method.try_init
    pub fn init(self) -> ValueObserver<T> {
        ValueObserver {
            instrument: self
                .meter
                .new_async_instrument(self.descriptor, self.runner)
                .unwrap(),
            _marker: std::marker::PhantomData,
        }
    }
}
