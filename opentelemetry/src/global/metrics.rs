use crate::metrics::{noop::NoopMeterProvider, Meter, MeterProvider};
use crate::{otel_info, InstrumentationScope};
use std::sync::Arc;

use arc_swap::ArcSwapOption;

type GlobalMeterProvider = Arc<dyn MeterProvider + Send + Sync>;

// ArcSwap requires a sized pointee.
struct MeterProviderWrapper {
    provider: GlobalMeterProvider,
}

/// The global `MeterProvider` singleton, lazily initialized with a no-op provider.
static GLOBAL_METER_PROVIDER: ArcSwapOption<MeterProviderWrapper> = ArcSwapOption::const_empty();

/// Runs `f` with the current global provider.
#[inline]
fn with_meter_provider<R>(f: impl FnOnce(&GlobalMeterProvider) -> R) -> R {
    let current = GLOBAL_METER_PROVIDER.load();
    if let Some(provider) = current.as_deref() {
        return f(&provider.provider);
    }
    drop(current);

    // Lazily install the no-op provider without overwriting a concurrent set.
    let noop = Arc::new(MeterProviderWrapper {
        provider: Arc::new(NoopMeterProvider::new()),
    });
    let previous = GLOBAL_METER_PROVIDER.compare_and_swap(&None::<Arc<_>>, Some(noop.clone()));
    match previous.as_deref() {
        Some(winner) => f(&winner.provider),
        None => f(&noop.provider),
    }
}

/// Sets the given [`MeterProvider`] instance as the current global meter
/// provider.
/// Libraries should NOT call this function. It is intended for applications/executables.
///
/// **NOTE:** This function should be called before getting [`Meter`] instances via [`meter()`] or [`meter_with_scope()`]. Otherwise, you could get no-op [`Meter`] instances.
pub fn set_meter_provider<P>(new_provider: P)
where
    P: MeterProvider + Send + Sync + 'static,
{
    GLOBAL_METER_PROVIDER.store(Some(Arc::new(MeterProviderWrapper {
        provider: Arc::new(new_provider),
    })));
    otel_info!(name: "MeterProvider.GlobalSet", message = "Global meter provider is set. Meters can now be created using global::meter() or global::meter_with_scope().");
}

/// Returns an instance of the currently configured global [`MeterProvider`].
pub fn meter_provider() -> GlobalMeterProvider {
    with_meter_provider(Arc::clone)
}

/// Creates a named [`Meter`] via the currently configured global [`MeterProvider`].
///
/// This is a more convenient way of expressing `global::meter_provider().meter(name)`.
///
/// **NOTE:** Calls to [`meter()`] return a [`Meter`] backed by the global [`MeterProvider`] configured during the method invocation.
/// If the global [`MeterProvider`] is changed after getting [`Meter`] instances from these calls, the [`Meter`] instances returned will not reflect the change.
pub fn meter(name: &'static str) -> Meter {
    with_meter_provider(|provider| provider.meter(name))
}

/// Creates a [`Meter`] with the given instrumentation scope.
///
/// This is a simpler alternative to `global::meter_provider().meter_with_scope(...)`
///
/// **NOTE:** Calls to [`meter_with_scope()`] return a [`Meter`] backed by the global [`MeterProvider`] configured during the method invocation.
/// If the global [`MeterProvider`] is changed after getting [`Meter`] instances from these calls, the [`Meter`] instances returned will not reflect the change.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use opentelemetry::global::meter_with_scope;
/// use opentelemetry::InstrumentationScope;
/// use opentelemetry::KeyValue;
///
/// let scope = InstrumentationScope::builder("io.opentelemetry")
///     .with_version("0.17")
///     .with_schema_url("https://opentelemetry.io/schemas/1.2.0")
///     .with_attributes(vec![(KeyValue::new("key", "value"))])
///     .build();
///
/// let meter = meter_with_scope(scope);
/// ```
pub fn meter_with_scope(scope: InstrumentationScope) -> Meter {
    with_meter_provider(|provider| provider.meter_with_scope(scope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct RecordingMeterProvider {
        scopes: Arc<Mutex<Vec<InstrumentationScope>>>,
    }

    impl MeterProvider for RecordingMeterProvider {
        fn meter_with_scope(&self, scope: InstrumentationScope) -> Meter {
            let current = GLOBAL_METER_PROVIDER.load();
            let current = current.as_ref().unwrap();
            assert_eq!(Arc::strong_count(current), 1);
            assert_eq!(Arc::strong_count(&current.provider), 1);
            self.scopes.lock().unwrap().push(scope.clone());
            NoopMeterProvider::new().meter_with_scope(scope)
        }
    }

    #[test]
    fn test_global_meter_provider() {
        meter("noop").u64_counter("counter").build().add(1, &[]);

        let scopes = Arc::new(Mutex::new(Vec::new()));
        set_meter_provider(RecordingMeterProvider {
            scopes: scopes.clone(),
        });
        meter("named");
        let scope = InstrumentationScope::builder("scoped")
            .with_version("1.0")
            .build();
        meter_with_scope(scope.clone());
        assert_eq!(
            *scopes.lock().unwrap(),
            vec![InstrumentationScope::builder("named").build(), scope]
        );

        let owned = meter_provider();
        with_meter_provider(|provider| {
            assert!(Arc::ptr_eq(provider, &owned));
            assert_eq!(Arc::strong_count(provider), 2);
            set_meter_provider(NoopMeterProvider::new());
        });
        assert_eq!(Arc::strong_count(&owned), 1);
        owned.meter("retained");
        assert_eq!(scopes.lock().unwrap().last().unwrap().name(), "retained");
        with_meter_provider(|provider| assert!(!Arc::ptr_eq(provider, &owned)));
    }
}
