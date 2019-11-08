use crate::api::metrics;
use crate::Key;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) type LabelSet = HashMap<Cow<'static, str>, Cow<'static, str>>;
impl metrics::LabelSet for LabelSet {}

fn convert_label_set(label_set: &LabelSet) -> HashMap<&str, &str> {
    label_set
        .iter()
        .map(|(key, value)| (key.as_ref(), value.as_ref()))
        .collect()
}

pub(crate) fn convert_labels(labels: &Vec<Key>) -> Vec<&str> {
    labels
        .iter()
        .map(|k| k.inner())
        .map(|k| k.as_ref())
        .collect()
}

// COUNTER COMPAT
#[derive(Clone)]
pub struct IntCounterHandle(prometheus::IntCounter);

impl metrics::Counter<i64, LabelSet> for prometheus::IntCounterVec {
    type Handle = IntCounterHandle;

    fn measurement(&self, value: i64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        IntCounterHandle(self.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for prometheus::IntCounterVec {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.with(&convert_label_set(label_set))
            .inc_by(value.into_i64())
    }
}

impl metrics::InstrumentHandle for IntCounterHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.inc_by(value.into_i64())
    }
}

impl metrics::counter::CounterHandle<i64> for IntCounterHandle {}

#[derive(Clone)]
pub struct CounterHandle(prometheus::Counter);

impl metrics::Counter<f64, LabelSet> for prometheus::CounterVec {
    type Handle = CounterHandle;

    fn measurement(&self, value: f64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        CounterHandle(self.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for prometheus::CounterVec {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.with(&convert_label_set(label_set))
            .inc_by(value.into_f64())
    }
}

impl metrics::InstrumentHandle for CounterHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.inc_by(value.into_f64())
    }
}

impl metrics::counter::CounterHandle<f64> for CounterHandle {}

// GAUGE COMPAT
#[derive(Clone)]
pub struct IntGaugeHandle(prometheus::IntGauge);

impl metrics::Gauge<i64, LabelSet> for prometheus::IntGaugeVec {
    type Handle = IntGaugeHandle;

    fn measurement(&self, value: i64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        IntGaugeHandle(self.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for prometheus::IntGaugeVec {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.with(&convert_label_set(label_set))
            .set(value.into_i64())
    }
}

impl metrics::InstrumentHandle for IntGaugeHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.set(value.into_i64())
    }
}

impl metrics::gauge::GaugeHandle<i64> for IntGaugeHandle {}

#[derive(Clone)]
pub struct GaugeHandle(prometheus::Gauge);

impl metrics::Gauge<f64, LabelSet> for prometheus::GaugeVec {
    type Handle = GaugeHandle;

    fn measurement(&self, value: f64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        GaugeHandle(self.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for prometheus::GaugeVec {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.with(&convert_label_set(label_set))
            .set(value.into_f64())
    }
}

impl metrics::InstrumentHandle for GaugeHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.set(value.into_f64())
    }
}

impl metrics::gauge::GaugeHandle<f64> for GaugeHandle {}

// MEASURE COMPAT
#[derive(Clone)]
pub struct IntMeasureHandle(prometheus::Histogram);

/// Prometheus Histograms do not have i64 variant, `IntMeasure` will convert i64 to float when it
/// records values.
#[derive(Clone)]
pub struct IntMeasure(prometheus::HistogramVec);

impl IntMeasure {
    pub(crate) fn new(histogram: prometheus::HistogramVec) -> Self {
        IntMeasure(histogram)
    }
}

impl metrics::Measure<i64, LabelSet> for IntMeasure {
    type Handle = IntMeasureHandle;

    fn measurement(&self, value: i64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        IntMeasureHandle(self.0.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for IntMeasure {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.0
            .with(&convert_label_set(label_set))
            .observe(value.into_i64() as f64)
    }
}

impl metrics::InstrumentHandle for IntMeasureHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.observe(value.into_i64() as f64)
    }
}

impl metrics::measure::MeasureHandle<i64> for IntMeasureHandle {}

#[derive(Clone)]
pub struct MeasureHandle(prometheus::Histogram);

impl metrics::Measure<f64, LabelSet> for prometheus::HistogramVec {
    type Handle = MeasureHandle;

    fn measurement(&self, value: f64) -> metrics::Measurement<LabelSet> {
        metrics::Measurement::new(
            Arc::new(self.clone()),
            metrics::MeasurementValue::from(value),
        )
    }

    fn acquire_handle(&self, labels: &LabelSet) -> Self::Handle {
        MeasureHandle(self.with(&convert_label_set(labels)))
    }
}

impl metrics::Instrument<LabelSet> for prometheus::HistogramVec {
    fn record_one(&self, value: metrics::MeasurementValue, label_set: &LabelSet) {
        self.with(&convert_label_set(label_set))
            .observe(value.into_f64())
    }
}

impl metrics::InstrumentHandle for MeasureHandle {
    fn record_one(&self, value: metrics::MeasurementValue) {
        self.0.observe(value.into_f64())
    }
}

impl metrics::measure::MeasureHandle<f64> for MeasureHandle {}
