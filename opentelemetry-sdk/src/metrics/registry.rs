//! Metrics Registry API
use crate::metrics::sdk_api::{Descriptor, SyncInstrumentCore};
use core::fmt;
use opentelemetry_api::{
    metrics::{MetricsError, Result},
    Context,
};
use std::sync::{Arc, Mutex};
use std::{any::Any, collections::HashMap};

use super::sdk_api::{AsyncInstrumentCore, InstrumentCore, MeterCore};

/// Create a new `UniqueInstrumentMeterCore` from a `InstrumentProvider`.
pub fn unique_instrument_meter_core<T>(core: T) -> UniqueInstrumentMeterCore
where
    T: AnyMeterCore + Send + Sync + 'static,
{
    UniqueInstrumentMeterCore::wrap(core)
}

/// An extension trait that allows meters to be downcast
pub trait AnyMeterCore: MeterCore {
    /// Returns the current type as [`Any`]
    fn as_any(&self) -> &dyn Any;
}

impl<T: MeterCore + 'static> AnyMeterCore for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Implements the [`MeterCore`] interface, adding uniqueness checking for
/// instrument descriptors.
pub struct UniqueInstrumentMeterCore {
    inner: Box<dyn AnyMeterCore + Send + Sync>,
    state: Mutex<HashMap<String, Arc<dyn InstrumentCore + Send + Sync>>>,
}

impl fmt::Debug for UniqueInstrumentMeterCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UniqueInstrumentMeterCore")
    }
}

impl UniqueInstrumentMeterCore {
    fn wrap<T>(inner: T) -> Self
    where
        T: AnyMeterCore + Send + Sync + 'static,
    {
        UniqueInstrumentMeterCore {
            inner: Box::new(inner),
            state: Mutex::new(HashMap::default()),
        }
    }

    pub(crate) fn meter_core(&self) -> &dyn Any {
        self.inner.as_any()
    }
}

impl MeterCore for UniqueInstrumentMeterCore {
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>> {
        self.state.lock().map_err(Into::into).and_then(|mut state| {
            let instrument = check_uniqueness(&state, &descriptor)?;
            match instrument {
                Some(instrument) => Ok(instrument),
                None => {
                    let instrument = self.inner.new_sync_instrument(descriptor.clone())?;
                    state.insert(descriptor.name().into(), instrument.clone().as_dyn_core());

                    Ok(instrument)
                }
            }
        })
    }

    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>> {
        self.state.lock().map_err(Into::into).and_then(|mut state| {
            let instrument = check_uniqueness(&state, &descriptor)?;
            match instrument {
                Some(instrument) => Ok(instrument),
                None => {
                    let instrument = self.inner.new_async_instrument(descriptor)?;
                    state.insert(
                        instrument.descriptor().name().into(),
                        instrument.clone().as_dyn_core(),
                    );

                    Ok(instrument)
                }
            }
        })
    }

    fn register_callback(&self, f: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()> {
        self.inner.register_callback(f)
    }
}

fn check_uniqueness<T: Clone + 'static>(
    instruments: &HashMap<String, Arc<dyn InstrumentCore + Send + Sync>>,
    descriptor: &Descriptor,
) -> Result<Option<T>> {
    if let Some(instrument) = instruments.get(descriptor.name()) {
        if is_equal(instrument.descriptor(), descriptor) {
            Ok(instrument.as_any().downcast_ref::<T>().cloned())
        } else {
            Err(MetricsError::MetricKindMismatch(format!(
                "metric {} registered as a {:?} {:?}",
                descriptor.name(),
                descriptor.number_kind(),
                descriptor.instrument_kind()
            )))
        }
    } else {
        Ok(None)
    }
}

fn is_equal(a: &Descriptor, b: &Descriptor) -> bool {
    a.instrument_kind() == b.instrument_kind() && a.number_kind() == b.number_kind()
}
