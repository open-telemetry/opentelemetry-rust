use core::fmt;
use std::{any::Any, borrow::Cow, collections::HashSet, sync::Arc};

use opentelemetry::{
    global,
    metrics::{
        noop::{NoopAsyncInstrument, NoopRegistration},
        AsyncInstrument, Callback, CallbackRegistration, Counter, Histogram, InstrumentProvider,
        MetricsError, ObservableCounter, ObservableGauge, ObservableUpDownCounter,
        Observer as ApiObserver, Result, Unit, UpDownCounter,
    },
    KeyValue,
};

use crate::instrumentation::Scope;
use crate::metrics::{
    instrument::{
        DefaultInstrument, Instrument, InstrumentKind, Observable, ObservableId, EMPTY_MEASURE_MSG,
    },
    internal::{self, Number},
    pipeline::{Pipelines, Resolver},
};

// maximum length of instrument name
const INSTRUMENT_NAME_MAX_LENGTH: usize = 255;
// maximum length of instrument unit name
const INSTRUMENT_UNIT_NAME_MAX_LENGTH: usize = 63;
const INSTRUMENT_NAME_ALLOWED_NON_ALPHANUMERIC_CHARS: [char; 4] = ['_', '.', '-', '/'];

// instrument validation error strings
const INSTRUMENT_NAME_EMPTY: &str = "instrument name must be non-empty";
const INSTRUMENT_NAME_LENGTH: &str = "instrument name must be less than 256 characters";
const INSTRUMENT_NAME_INVALID_CHAR: &str =
    "characters in instrument name must be ASCII and belong to the alphanumeric characters, '_', '.', '-' and '/'";
const INSTRUMENT_NAME_FIRST_ALPHABETIC: &str =
    "instrument name must start with an alphabetic character";
const INSTRUMENT_UNIT_LENGTH: &str = "instrument unit must be less than 64 characters";
const INSTRUMENT_UNIT_INVALID_CHAR: &str = "characters in instrument unit must be ASCII";

/// Handles the creation and coordination of all metric instruments.
///
/// A meter represents a single instrumentation scope; all metric telemetry
/// produced by an instrumentation scope will use metric instruments from a
/// single meter.
///
/// See the [Meter API] docs for usage.
///
/// [Meter API]: opentelemetry::metrics::Meter
pub struct DefaultMeter {
    scope: Scope,
    pipes: Arc<Pipelines>,
    u64_resolver: Resolver<u64>,
    i64_resolver: Resolver<i64>,
    f64_resolver: Resolver<f64>,
    validation_policy: InstrumentValidationPolicy,
}

impl DefaultMeter {
    pub(crate) fn new(scope: Scope, pipes: Arc<Pipelines>) -> Self {
        let view_cache = Default::default();

        DefaultMeter {
            scope,
            pipes: Arc::clone(&pipes),
            u64_resolver: Resolver::new(Arc::clone(&pipes), Arc::clone(&view_cache)),
            i64_resolver: Resolver::new(Arc::clone(&pipes), Arc::clone(&view_cache)),
            f64_resolver: Resolver::new(pipes, view_cache),
            validation_policy: InstrumentValidationPolicy::HandleGlobalAndIgnore,
        }
    }

    #[cfg(test)]
    fn with_validation_policy(self, validation_policy: InstrumentValidationPolicy) -> Self {
        Self {
            validation_policy,
            ..self
        }
    }
}

#[doc(hidden)]
impl InstrumentProvider for DefaultMeter {
    fn u64_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Counter<u64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.u64_resolver);
        p.lookup(
            InstrumentKind::Counter,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| Counter::new(Arc::new(i)))
    }

    fn f64_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Counter<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::Counter,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| Counter::new(Arc::new(i)))
    }

    fn u64_observable_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<u64>>,
    ) -> Result<ObservableCounter<u64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.u64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableCounter::new(observable))
    }

    fn f64_observable_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<f64>>,
    ) -> Result<ObservableCounter<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }
        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableCounter::new(observable))
    }

    fn i64_up_down_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<UpDownCounter<i64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.i64_resolver);
        p.lookup(
            InstrumentKind::UpDownCounter,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| UpDownCounter::new(Arc::new(i)))
    }

    fn f64_up_down_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<UpDownCounter<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::UpDownCounter,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| UpDownCounter::new(Arc::new(i)))
    }

    fn i64_observable_up_down_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<i64>>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.i64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableUpDownCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableUpDownCounter::new(Arc::new(
                NoopAsyncInstrument::new(),
            )));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableUpDownCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableUpDownCounter::new(observable))
    }

    fn f64_observable_up_down_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<f64>>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableUpDownCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableUpDownCounter::new(Arc::new(
                NoopAsyncInstrument::new(),
            )));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableUpDownCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableUpDownCounter::new(observable))
    }

    fn u64_observable_gauge(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<u64>>,
    ) -> Result<ObservableGauge<u64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.u64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableGauge::new(observable))
    }

    fn i64_observable_gauge(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<i64>>,
    ) -> Result<ObservableGauge<i64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.i64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableGauge::new(observable))
    }

    fn f64_observable_gauge(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
        callbacks: Vec<Callback<f64>>,
    ) -> Result<ObservableGauge<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        let ms = p.measures(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if ms.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            ms,
        ));

        for callback in callbacks {
            let cb_inst = Arc::clone(&observable);
            self.pipes
                .register_callback(move || callback(cb_inst.as_ref()));
        }

        Ok(ObservableGauge::new(observable))
    }

    fn f64_histogram(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Histogram<f64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.f64_resolver);
        p.lookup(
            InstrumentKind::Histogram,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| Histogram::new(Arc::new(i)))
    }

    fn u64_histogram(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Histogram<u64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.u64_resolver);
        p.lookup(
            InstrumentKind::Histogram,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| Histogram::new(Arc::new(i)))
    }

    fn i64_histogram(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Histogram<i64>> {
        validate_instrument_config(name.as_ref(), unit.as_ref(), self.validation_policy)?;
        let p = InstProvider::new(self, &self.i64_resolver);

        p.lookup(
            InstrumentKind::Histogram,
            name,
            description,
            unit.unwrap_or_default(),
        )
        .map(|i| Histogram::new(Arc::new(i)))
    }

    fn register_callback(
        &self,
        insts: &[Arc<dyn Any>],
        callback: Box<dyn Fn(&dyn ApiObserver) + Send + Sync>,
    ) -> Result<Box<dyn CallbackRegistration>> {
        if insts.is_empty() {
            return Ok(Box::new(NoopRegistration::new()));
        }

        let mut reg = Observer::default();
        let mut errs = vec![];
        for inst in insts {
            if let Some(i64_obs) = inst.downcast_ref::<Observable<i64>>() {
                if let Err(err) = i64_obs.registerable(&self.scope) {
                    if !err.to_string().contains(EMPTY_MEASURE_MSG) {
                        errs.push(err);
                    }
                    continue;
                }
                reg.register_i64(i64_obs.id.clone());
            } else if let Some(u64_obs) = inst.downcast_ref::<Observable<u64>>() {
                if let Err(err) = u64_obs.registerable(&self.scope) {
                    if !err.to_string().contains(EMPTY_MEASURE_MSG) {
                        errs.push(err);
                    }
                    continue;
                }
                reg.register_u64(u64_obs.id.clone());
            } else if let Some(f64_obs) = inst.downcast_ref::<Observable<f64>>() {
                if let Err(err) = f64_obs.registerable(&self.scope) {
                    if !err.to_string().contains(EMPTY_MEASURE_MSG) {
                        errs.push(err);
                    }
                    continue;
                }
                reg.register_f64(f64_obs.id.clone());
            } else {
                // Instrument external to the SDK.
                return Err(MetricsError::Other(
                    "invalid observable: from different implementation".into(),
                ));
            }
        }

        if !errs.is_empty() {
            return Err(MetricsError::Other(format!("{errs:?}")));
        }

        if reg.is_empty() {
            // All instruments use drop aggregation or are invalid.
            return Ok(Box::new(NoopRegistration::new()));
        }

        self.pipes.register_multi_callback(move || callback(&reg))
    }
}

/// Validation policy for instrument
#[derive(Clone, Copy)]
enum InstrumentValidationPolicy {
    HandleGlobalAndIgnore,
    /// Currently only for test
    #[cfg(test)]
    Strict,
}

fn validate_instrument_config(
    name: &str,
    unit: Option<&Unit>,
    policy: InstrumentValidationPolicy,
) -> Result<()> {
    match validate_instrument_name(name).and_then(|_| validate_instrument_unit(unit)) {
        Ok(_) => Ok(()),
        Err(err) => match policy {
            InstrumentValidationPolicy::HandleGlobalAndIgnore => {
                global::handle_error(err);
                Ok(())
            }
            #[cfg(test)]
            InstrumentValidationPolicy::Strict => Err(err),
        },
    }
}

fn validate_instrument_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_EMPTY,
        ));
    }
    if name.len() > INSTRUMENT_NAME_MAX_LENGTH {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_LENGTH,
        ));
    }
    if name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_FIRST_ALPHABETIC,
        ));
    }
    if name.contains(|c: char| {
        !c.is_ascii_alphanumeric() && !INSTRUMENT_NAME_ALLOWED_NON_ALPHANUMERIC_CHARS.contains(&c)
    }) {
        return Err(MetricsError::InvalidInstrumentConfiguration(
            INSTRUMENT_NAME_INVALID_CHAR,
        ));
    }
    Ok(())
}

fn validate_instrument_unit(unit: Option<&Unit>) -> Result<()> {
    if let Some(unit) = unit {
        if unit.as_str().len() > INSTRUMENT_UNIT_NAME_MAX_LENGTH {
            return Err(MetricsError::InvalidInstrumentConfiguration(
                INSTRUMENT_UNIT_LENGTH,
            ));
        }
        if unit.as_str().contains(|c: char| !c.is_ascii()) {
            return Err(MetricsError::InvalidInstrumentConfiguration(
                INSTRUMENT_UNIT_INVALID_CHAR,
            ));
        }
    }
    Ok(())
}

#[derive(Default)]
struct Observer {
    f64s: HashSet<ObservableId<f64>>,
    i64s: HashSet<ObservableId<i64>>,
    u64s: HashSet<ObservableId<u64>>,
}

impl Observer {
    fn is_empty(&self) -> bool {
        self.f64s.is_empty() && self.i64s.is_empty() && self.u64s.is_empty()
    }

    pub(crate) fn register_i64(&mut self, id: ObservableId<i64>) {
        self.i64s.insert(id);
    }

    pub(crate) fn register_f64(&mut self, id: ObservableId<f64>) {
        self.f64s.insert(id);
    }

    pub(crate) fn register_u64(&mut self, id: ObservableId<u64>) {
        self.u64s.insert(id);
    }
}

impl ApiObserver for Observer {
    fn observe_f64(&self, inst: &dyn AsyncInstrument<f64>, measurement: f64, attrs: &[KeyValue]) {
        if let Some(f64_obs) = inst.as_any().downcast_ref::<Observable<f64>>() {
            if self.f64s.contains(&f64_obs.id) {
                f64_obs.observe(measurement, attrs)
            } else {
                global::handle_error(
                    MetricsError::Other(format!("observable instrument not registered for callback, failed to record. name: {}, description: {}, unit: {:?}, number: f64",
                    f64_obs.id.inner.name,
                    f64_obs.id.inner.description,
                    f64_obs.id.inner.unit,
                )))
            }
        } else {
            global::handle_error(MetricsError::Other(
                "unknown observable instrument, failed to record.".into(),
            ))
        }
    }

    fn observe_u64(&self, inst: &dyn AsyncInstrument<u64>, measurement: u64, attrs: &[KeyValue]) {
        if let Some(u64_obs) = inst.as_any().downcast_ref::<Observable<u64>>() {
            if self.u64s.contains(&u64_obs.id) {
                u64_obs.observe(measurement, attrs)
            } else {
                global::handle_error(
                    MetricsError::Other(format!("observable instrument not registered for callback, failed to record. name: {}, description: {}, unit: {:?}, number: f64",
                    u64_obs.id.inner.name,
                    u64_obs.id.inner.description,
                    u64_obs.id.inner.unit,
                )))
            }
        } else {
            global::handle_error(MetricsError::Other(
                "unknown observable instrument, failed to record.".into(),
            ))
        }
    }

    fn observe_i64(&self, inst: &dyn AsyncInstrument<i64>, measurement: i64, attrs: &[KeyValue]) {
        if let Some(i64_obs) = inst.as_any().downcast_ref::<Observable<i64>>() {
            if self.i64s.contains(&i64_obs.id) {
                i64_obs.observe(measurement, attrs)
            } else {
                global::handle_error(
                    MetricsError::Other(format!("observable instrument not registered for callback, failed to record. name: {}, description: {}, unit: {:?}, number: f64",
                    i64_obs.id.inner.name,
                    i64_obs.id.inner.description,
                    i64_obs.id.inner.unit,
                )))
            }
        } else {
            global::handle_error(MetricsError::Other(
                "unknown observable instrument, failed to record.".into(),
            ))
        }
    }
}

impl fmt::Debug for DefaultMeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter").field("scope", &self.scope).finish()
    }
}

/// Provides all OpenTelemetry instruments.
struct InstProvider<'a, T> {
    meter: &'a DefaultMeter,
    resolve: &'a Resolver<T>,
}

impl<'a, T> InstProvider<'a, T>
where
    T: Number<T>,
{
    fn new(meter: &'a DefaultMeter, resolve: &'a Resolver<T>) -> Self {
        InstProvider { meter, resolve }
    }

    /// lookup returns the resolved InstrumentImpl.
    fn lookup(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Unit,
    ) -> Result<DefaultInstrument<T>> {
        let aggregators = self.measures(kind, name, description, unit)?;
        Ok(DefaultInstrument {
            measures: aggregators,
        })
    }

    fn measures(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Unit,
    ) -> Result<Vec<Arc<dyn internal::Measure<T>>>> {
        let inst = Instrument {
            name,
            description: description.unwrap_or_default(),
            unit,
            kind: Some(kind),
            scope: self.meter.scope.clone(),
        };

        self.resolve.measures(inst)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentelemetry::metrics::{InstrumentProvider, MetricsError, Unit};

    use super::{
        DefaultMeter, InstrumentValidationPolicy, INSTRUMENT_NAME_FIRST_ALPHABETIC,
        INSTRUMENT_NAME_INVALID_CHAR, INSTRUMENT_NAME_LENGTH, INSTRUMENT_UNIT_INVALID_CHAR,
        INSTRUMENT_UNIT_LENGTH,
    };
    use crate::{metrics::pipeline::Pipelines, Resource, Scope};

    #[test]
    fn test_instrument_config_validation() {
        // scope and pipelines are not related to test
        let meter = DefaultMeter::new(
            Scope::default(),
            Arc::new(Pipelines::new(Resource::default(), Vec::new(), Vec::new())),
        )
        .with_validation_policy(InstrumentValidationPolicy::Strict);
        // (name, expected error)
        let instrument_name_test_cases = vec![
            ("validateName", ""),
            ("_startWithNoneAlphabet", INSTRUMENT_NAME_FIRST_ALPHABETIC),
            ("utf8char锈", INSTRUMENT_NAME_INVALID_CHAR),
            ("a".repeat(255).leak(), ""),
            ("a".repeat(256).leak(), INSTRUMENT_NAME_LENGTH),
            ("invalid name", INSTRUMENT_NAME_INVALID_CHAR),
            // hyphens are now valid characters in the specification.
            // https://github.com/open-telemetry/opentelemetry-specification/pull/3684
            ("allow/hyphen", ""),
        ];
        for (name, expected_error) in instrument_name_test_cases {
            let assert = |result: Result<_, MetricsError>| {
                if expected_error.is_empty() {
                    assert!(result.is_ok());
                } else {
                    assert!(matches!(
                        result.unwrap_err(),
                        MetricsError::InvalidInstrumentConfiguration(msg) if msg == expected_error
                    ));
                }
            };

            assert(meter.u64_counter(name.into(), None, None).map(|_| ()));
            assert(meter.f64_counter(name.into(), None, None).map(|_| ()));
            assert(
                meter
                    .u64_observable_counter(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_counter(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_up_down_counter(name.into(), None, None)
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_up_down_counter(name.into(), None, None)
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_observable_up_down_counter(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_up_down_counter(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .u64_observable_gauge(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_observable_gauge(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_gauge(name.into(), None, None, Vec::new())
                    .map(|_| ()),
            );
            assert(meter.f64_histogram(name.into(), None, None).map(|_| ()));
            assert(meter.u64_histogram(name.into(), None, None).map(|_| ()));
            assert(meter.i64_histogram(name.into(), None, None).map(|_| ()));
        }

        // (unit, expected error)
        let instrument_unit_test_cases = vec![
            (
                "0123456789012345678901234567890123456789012345678901234567890123",
                INSTRUMENT_UNIT_LENGTH,
            ),
            ("utf8char锈", INSTRUMENT_UNIT_INVALID_CHAR),
            ("kb", ""),
        ];

        for (unit, expected_error) in instrument_unit_test_cases {
            let assert = |result: Result<_, MetricsError>| {
                if expected_error.is_empty() {
                    assert!(result.is_ok());
                } else {
                    assert!(matches!(
                        result.unwrap_err(),
                        MetricsError::InvalidInstrumentConfiguration(msg) if msg == expected_error
                    ));
                }
            };
            let unit = Some(Unit::new(unit));
            assert(
                meter
                    .u64_counter("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_counter("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .u64_observable_counter("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_counter("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_up_down_counter("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_up_down_counter("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_observable_up_down_counter("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_up_down_counter("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .u64_observable_gauge("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_observable_gauge("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_observable_gauge("test".into(), None, unit.clone(), Vec::new())
                    .map(|_| ()),
            );
            assert(
                meter
                    .f64_histogram("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .u64_histogram("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
            assert(
                meter
                    .i64_histogram("test".into(), None, unit.clone())
                    .map(|_| ()),
            );
        }
    }
}
