use crate::metrics::sdk_api::MeterCore;
use crate::metrics::sdk_api::{
    AsyncInstrumentCore, Descriptor, InstrumentKind, Number, NumberKind, SyncInstrumentCore,
};
use opentelemetry_api::metrics::{
    AsyncCounter, AsyncUpDownCounter, ObservableUpDownCounter, SyncCounter, SyncHistogram,
    SyncUpDownCounter, UpDownCounter,
};
use opentelemetry_api::KeyValue;
use opentelemetry_api::{
    metrics::{
        AsyncGauge, Counter, Histogram, InstrumentProvider, Meter, ObservableCounter,
        ObservableGauge, Result, Unit,
    },
    Context, InstrumentationLibrary,
};
use std::sync::Arc;

/// wraps impl to be a full implementation of a Meter.
pub fn wrap_meter_core(
    core: Arc<dyn MeterCore + Send + Sync>,
    library: InstrumentationLibrary,
) -> Meter {
    Meter::new(library, Arc::new(MeterImpl(core)))
}

struct MeterImpl(Arc<dyn MeterCore + Send + Sync>);

struct SyncInstrument(Arc<dyn SyncInstrumentCore + Send + Sync>);

impl<T: Into<Number>> SyncCounter<T> for SyncInstrument {
    fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.record_one(cx, value.into(), attributes)
    }
}

impl<T: Into<Number>> SyncUpDownCounter<T> for SyncInstrument {
    fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.record_one(cx, value.into(), attributes)
    }
}

impl<T: Into<Number>> SyncHistogram<T> for SyncInstrument {
    fn record(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.record_one(cx, value.into(), attributes)
    }
}

struct AsyncInstrument(Arc<dyn AsyncInstrumentCore + Send + Sync>);

impl<T: Into<Number>> AsyncCounter<T> for AsyncInstrument {
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe_one(cx, value.into(), attributes)
    }
}

impl<T: Into<Number>> AsyncUpDownCounter<T> for AsyncInstrument {
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe_one(cx, value.into(), attributes)
    }
}

impl<T: Into<Number>> AsyncGauge<T> for AsyncInstrument {
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe_one(cx, value.into(), attributes)
    }
}

impl InstrumentProvider for MeterImpl {
    fn u64_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<Counter<u64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::Counter,
            NumberKind::U64,
            description,
            unit,
        ))?;

        Ok(Counter::new(Arc::new(SyncInstrument(instrument))))
    }

    fn f64_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<Counter<f64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::Counter,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(Counter::new(Arc::new(SyncInstrument(instrument))))
    }

    fn u64_observable_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableCounter<u64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::Counter,
            NumberKind::U64,
            description,
            unit,
        ))?;

        Ok(ObservableCounter::new(Arc::new(AsyncInstrument(
            instrument,
        ))))
    }

    fn f64_observable_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableCounter<f64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::Counter,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(ObservableCounter::new(Arc::new(AsyncInstrument(
            instrument,
        ))))
    }

    fn i64_up_down_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<UpDownCounter<i64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::UpDownCounter,
            NumberKind::I64,
            description,
            unit,
        ))?;

        Ok(UpDownCounter::new(Arc::new(SyncInstrument(instrument))))
    }

    fn f64_up_down_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<UpDownCounter<f64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::UpDownCounter,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(UpDownCounter::new(Arc::new(SyncInstrument(instrument))))
    }

    fn i64_observable_up_down_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::UpDownCounterObserver,
            NumberKind::I64,
            description,
            unit,
        ))?;

        Ok(ObservableUpDownCounter::new(Arc::new(AsyncInstrument(
            instrument,
        ))))
    }

    fn f64_observable_up_down_counter(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::UpDownCounterObserver,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(ObservableUpDownCounter::new(Arc::new(AsyncInstrument(
            instrument,
        ))))
    }

    fn u64_observable_gauge(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableGauge<u64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::GaugeObserver,
            NumberKind::U64,
            description,
            unit,
        ))?;

        Ok(ObservableGauge::new(Arc::new(AsyncInstrument(instrument))))
    }

    fn i64_observable_gauge(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableGauge<i64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::GaugeObserver,
            NumberKind::I64,
            description,
            unit,
        ))?;

        Ok(ObservableGauge::new(Arc::new(AsyncInstrument(instrument))))
    }

    fn f64_observable_gauge(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<ObservableGauge<f64>> {
        let instrument = self.0.new_async_instrument(Descriptor::new(
            name,
            InstrumentKind::GaugeObserver,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(ObservableGauge::new(Arc::new(AsyncInstrument(instrument))))
    }

    fn f64_histogram(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<Histogram<f64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::Histogram,
            NumberKind::F64,
            description,
            unit,
        ))?;

        Ok(Histogram::new(Arc::new(SyncInstrument(instrument))))
    }

    fn u64_histogram(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<Histogram<u64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::Histogram,
            NumberKind::U64,
            description,
            unit,
        ))?;

        Ok(Histogram::new(Arc::new(SyncInstrument(instrument))))
    }

    fn i64_histogram(
        &self,
        name: String,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Result<Histogram<i64>> {
        let instrument = self.0.new_sync_instrument(Descriptor::new(
            name,
            InstrumentKind::Histogram,
            NumberKind::I64,
            description,
            unit,
        ))?;

        Ok(Histogram::new(Arc::new(SyncInstrument(instrument))))
    }

    fn register_callback(&self, callback: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()> {
        self.0.register_callback(callback)
    }
}
