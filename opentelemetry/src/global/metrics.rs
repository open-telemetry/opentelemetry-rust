use crate::metrics::{self, Meter, MeterProvider};
use crate::KeyValue;
use once_cell::sync::Lazy;
use std::{
    borrow::Cow,
    sync::{Arc, RwLock},
};

type GlobalMeterProvider = Arc<dyn MeterProvider + Send + Sync>;

/// The global `MeterProvider` singleton.
static GLOBAL_METER_PROVIDER: Lazy<RwLock<GlobalMeterProvider>> =
    Lazy::new(|| RwLock::new(Arc::new(metrics::noop::NoopMeterProvider::new())));

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
pub fn meter(name: impl Into<Cow<'static, str>>) -> Meter {
    meter_provider().meter(name.into().into_owned())
}

/// Creates a [`Meter`] with the name, version and schema url.
///
/// - name SHOULD uniquely identify the instrumentation scope, such as the instrumentation library (e.g. io.opentelemetry.contrib.mongodb), package, module or class name.
/// - version specifies the version of the instrumentation scope if the scope has a version
/// - schema url specifies the Schema URL that should be recorded in the emitted telemetry.
///
/// This is a convenient way of `global::meter_provider().versioned_meter(...)`
///
/// # Example
///
/// ```
/// use opentelemetry::global::meter_with_version;
/// use opentelemetry::KeyValue;
///
/// let meter = meter_with_version(
///     "io.opentelemetry",
///     Some("0.17"),
///     Some("https://opentelemetry.io/schemas/1.2.0"),
///     Some(vec![KeyValue::new("key", "value")]),
/// );
/// ```
pub fn meter_with_version(
    name: impl Into<Cow<'static, str>>,
    version: Option<impl Into<Cow<'static, str>>>,
    schema_url: Option<impl Into<Cow<'static, str>>>,
    attributes: Option<Vec<KeyValue>>,
) -> Meter {
    meter_provider().versioned_meter(
        name.into().into_owned(),
        version.map(|v| v.into().into_owned()),
        schema_url.map(|s| s.into().into_owned()),
        attributes,
    )
}
