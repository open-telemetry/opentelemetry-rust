use crate::metrics::{self, Meter, MeterProvider};
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    /// The global `Meter` provider singleton.
    static ref GLOBAL_METER_PROVIDER: RwLock<GlobalMeterProvider> = RwLock::new(GlobalMeterProvider::new(metrics::noop::NoopMeterProvider::new()));
}

/// Represents the globally configured [`MeterProvider`] instance for this
/// application.
#[derive(Debug, Clone)]
pub struct GlobalMeterProvider {
    provider: Arc<dyn MeterProvider + Send + Sync>,
}

impl MeterProvider for GlobalMeterProvider {
    fn meter(
        &self,
        name: &'static str,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> Meter {
        self.provider.meter(name, version, schema_url)
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
/// This is a more convenient way of expressing `global::meter_provider().meter(name, None, None)`.
pub fn meter(name: &'static str) -> Meter {
    meter_provider().meter(name, None, None)
}

/// Creates a [`Meter`] with the name, version and schema url.
///
/// - name SHOULD uniquely identify the instrumentation scope, such as the instrumentation library (e.g. io.opentelemetry.contrib.mongodb), package, module or class name.
/// - version specifies the version of the instrumentation scope if the scope has a version
/// - schema url specifies the Schema URL that should be recorded in the emitted telemetry.
///
/// This is a convenient way of `global::meter_provider().meter(...)`
/// # Example
/// ```rust
/// use opentelemetry_api::global::meter_with_version;
/// let meter = meter_with_version("io.opentelemetry", Some("0.17"), Some("https://opentelemetry.io/schemas/1.2.0"));
/// ```
///
pub fn meter_with_version(
    name: &'static str,
    version: Option<&'static str>,
    schema_url: Option<&'static str>,
) -> Meter {
    meter_provider().meter(name, version, schema_url)
}
