use crate::metrics::{self, Meter, MeterProvider};
use crate::InstrumentationLibrary;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

type GlobalMeterProvider = Arc<dyn MeterProvider + Send + Sync>;

/// The global `MeterProvider` singleton.
static GLOBAL_METER_PROVIDER: Lazy<RwLock<GlobalMeterProvider>> =
    Lazy::new(|| RwLock::new(Arc::new(crate::metrics::noop::NoopMeterProvider::new())));

/// Sets the given [`MeterProvider`] instance as the current global meter
/// provider.
pub fn set_meter_provider<P>(new_provider: P)
where
    P: metrics::MeterProvider + Send + Sync + 'static,
{
    let mut global_provider = GLOBAL_METER_PROVIDER
        .write()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned");
    *global_provider = Arc::new(new_provider);
}

/// Returns an instance of the currently configured global [`MeterProvider`].
pub fn meter_provider() -> GlobalMeterProvider {
    GLOBAL_METER_PROVIDER
        .read()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named [`Meter`] via the currently configured global [`MeterProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::meter_provider().meter(name)`.
pub fn meter(name: &'static str) -> Meter {
    meter_provider().meter(name)
}

/// Creates a [`Meter`] with the given instrumentation library.
///
/// This is a shortcut `global::meter_provider().library_meter(...)`
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use opentelemetry::global::library_meter;
/// use opentelemetry::InstrumentationLibrary;
/// use opentelemetry::KeyValue;
///
/// let library = Arc::new(
///     InstrumentationLibrary::builder("io.opentelemetry")
///         .with_version("0.17")
///         .with_schema_url("https://opentelemetry.io/schema/1.2.0")
///         .with_attributes(vec![(KeyValue::new("key", "value"))])
///         .build(),
/// );
///
/// let meter = library_meter(library);
/// ```
pub fn library_meter(library: Arc<InstrumentationLibrary>) -> Meter {
    meter_provider().library_meter(library)
}
