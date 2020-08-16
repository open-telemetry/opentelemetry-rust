use crate::api::metrics::{self, Meter, MeterProvider};
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    /// The global `Meter` provider singleton.
    static ref GLOBAL_METER_PROVIDER: RwLock<GlobalMeterProvider> = RwLock::new(GlobalMeterProvider::new(metrics::noop::NoopMeterProvider));
}

/// Represents the globally configured [`MeterProvider`] instance for this
/// application.
///
/// [`MeterProvider`]: ../../api/metrics/meter/trait.MeterProvider.html
#[derive(Debug, Clone)]
pub struct GlobalMeterProvider {
    provider: Arc<dyn MeterProvider + Send + Sync>,
}

impl MeterProvider for GlobalMeterProvider {
    fn meter(&self, name: &str) -> Meter {
        self.provider.meter(name)
    }
}

impl GlobalMeterProvider {
    /// Create a new global meter provider
    pub fn new<P>(provider: P) -> Self
    where
        P: MeterProvider + Send + Sync + 'static,
    {
        GlobalMeterProvider {
            provider: Arc::new(provider),
        }
    }
}

/// Sets the given [`MeterProvider`] instance as the current global meter
/// provider.
///
/// [`MeterProvider`]: ../../api/metrics/meter/trait.MeterProvider.html
pub fn set_meter_provider<P>(new_provider: P)
where
    P: metrics::MeterProvider + Send + Sync + 'static,
{
    let mut global_provider = GLOBAL_METER_PROVIDER
        .write()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned");
    *global_provider = GlobalMeterProvider::new(new_provider);
}

/// Returns an instance of the currently configured global [`MeterProvider`]
/// through [`GlobalMeterProvider`].
///
/// [`MeterProvider`]: ../../api/metrics/meter/trait.MeterProvider.html
/// [`GlobalMeterProvider`]: struct.GlobalMeterProvider.html
pub fn meter_provider() -> GlobalMeterProvider {
    GLOBAL_METER_PROVIDER
        .read()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named [`Meter`] via the configured [`GlobalMeterProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::meter_provider().meter(name)`.
///
/// [`Meter`]: ../../api/metrics/meter/struct.Meter.html
/// [`GlobalMeterProvider`]: struct.GlobalMeterProvider.html
pub fn meter(name: &str) -> Meter {
    meter_provider().meter(name)
}
