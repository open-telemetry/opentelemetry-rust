use core::fmt;
use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use opentelemetry_api::{
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
        Instrument, InstrumentImpl, InstrumentKind, Observable, ObservableId, StreamId,
        EMPTY_AGG_MSG,
    },
    internal::{self, Number},
    pipeline::{Pipelines, Resolver},
};

/// Handles the creation and coordination of all metric instruments.
///
/// A meter represents a single instrumentation scope; all metric telemetry
/// produced by an instrumentation scope will use metric instruments from a
/// single meter.
///
/// See the [Meter API] docs for usage.
///
/// [Meter API]: opentelemetry_api::metrics::Meter
pub struct Meter {
    scope: Scope,
    pipes: Arc<Pipelines>,
    u64_inst_provider: InstProvider<u64>,
    i64_inst_provider: InstProvider<i64>,
    f64_inst_provider: InstProvider<f64>,
}

impl Meter {
    pub(crate) fn new(scope: Scope, pipes: Arc<Pipelines>) -> Self {
        let view_cache = Default::default();

        Meter {
            scope: scope.clone(),
            pipes: Arc::clone(&pipes),
            u64_inst_provider: InstProvider::new(
                scope.clone(),
                Arc::clone(&pipes),
                Arc::clone(&view_cache),
            ),
            i64_inst_provider: InstProvider::new(
                scope.clone(),
                Arc::clone(&pipes),
                Arc::clone(&view_cache),
            ),
            f64_inst_provider: InstProvider::new(scope, pipes, view_cache),
        }
    }
}

#[doc(hidden)]
impl InstrumentProvider for Meter {
    fn u64_counter(
        &self,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Option<Unit>,
    ) -> Result<Counter<u64>> {
        self.u64_inst_provider
            .lookup(
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
        self.f64_inst_provider
            .lookup(
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
        let aggs = self.u64_inst_provider.aggregators(
            InstrumentKind::ObservableCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            aggs,
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
        let aggs = self.f64_inst_provider.aggregators(
            InstrumentKind::ObservableCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
            return Ok(ObservableCounter::new(Arc::new(NoopAsyncInstrument::new())));
        }
        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableCounter,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            aggs,
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
        self.i64_inst_provider
            .lookup(
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
        self.f64_inst_provider
            .lookup(
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
        let aggs = self.i64_inst_provider.aggregators(
            InstrumentKind::ObservableUpDownCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
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
            aggs,
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
        let aggs = self.f64_inst_provider.aggregators(
            InstrumentKind::ObservableUpDownCounter,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
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
            aggs,
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
        let aggs = self.u64_inst_provider.aggregators(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            aggs,
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
        let aggs = self.i64_inst_provider.aggregators(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            aggs,
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
        let aggs = self.f64_inst_provider.aggregators(
            InstrumentKind::ObservableGauge,
            name.clone(),
            description.clone(),
            unit.clone().unwrap_or_default(),
        )?;
        if aggs.is_empty() {
            return Ok(ObservableGauge::new(Arc::new(NoopAsyncInstrument::new())));
        }

        let observable = Arc::new(Observable::new(
            self.scope.clone(),
            InstrumentKind::ObservableGauge,
            name,
            description.unwrap_or_default(),
            unit.unwrap_or_default(),
            aggs,
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
        self.f64_inst_provider
            .lookup(
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
        self.u64_inst_provider
            .lookup(
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
        self.i64_inst_provider
            .lookup(
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
                    if !err.to_string().contains(EMPTY_AGG_MSG) {
                        errs.push(err);
                    }
                    continue;
                }
                reg.register_i64(i64_obs.id.clone());
            } else if let Some(u64_obs) = inst.downcast_ref::<Observable<u64>>() {
                if let Err(err) = u64_obs.registerable(&self.scope) {
                    if !err.to_string().contains(EMPTY_AGG_MSG) {
                        errs.push(err);
                    }
                    continue;
                }
                reg.register_u64(u64_obs.id.clone());
            } else if let Some(f64_obs) = inst.downcast_ref::<Observable<f64>>() {
                if let Err(err) = f64_obs.registerable(&self.scope) {
                    if !err.to_string().contains(EMPTY_AGG_MSG) {
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
            // All instruments use drop aggregation.
            return Ok(Box::new(NoopRegistration::new()));
        }

        self.pipes.register_multi_callback(move || callback(&reg))
    }
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

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter").field("scope", &self.scope).finish()
    }
}

/// Provides all OpenTelemetry instruments.
struct InstProvider<T> {
    scope: Scope,
    resolve: Resolver<T>,
}

impl<T> InstProvider<T>
where
    T: Number<T>,
{
    fn new(
        scope: Scope,
        pipes: Arc<Pipelines>,
        cache: Arc<Mutex<HashMap<Cow<'static, str>, StreamId>>>,
    ) -> Self {
        InstProvider {
            scope,
            resolve: Resolver::new(pipes, cache),
        }
    }

    /// lookup returns the resolved InstrumentImpl.
    fn lookup(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Unit,
    ) -> Result<InstrumentImpl<T>> {
        let aggregators = self.aggregators(kind, name, description, unit)?;
        Ok(InstrumentImpl { aggregators })
    }

    fn aggregators(
        &self,
        kind: InstrumentKind,
        name: Cow<'static, str>,
        description: Option<Cow<'static, str>>,
        unit: Unit,
    ) -> Result<Vec<Arc<dyn internal::Aggregator<T>>>> {
        let inst = Instrument {
            name,
            description: description.unwrap_or_default(),
            unit,
            kind: Some(kind),
            scope: self.scope.clone(),
        };

        self.resolve.aggregators(inst)
    }
}
