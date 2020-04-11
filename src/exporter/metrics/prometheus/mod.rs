//! # OpenTelemetry Prometheus Exporter
//!
//! This exporter currently delegates to the [Prometheus library]
//! library which implements the [Prometheus API].
//!
//! [Prometheus library]: https://github.com/tikv/rust-prometheus
//! [Prometheus API]: https://prometheus.io
use crate::api;
use crate::sdk;
use api::Key;
pub use prometheus::{
    default_registry, gather, Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram,
    HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
    TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Convert from `sdk::LabelSet` to `prometheus`' label format.
fn convert_label_set(label_set: &sdk::LabelSet) -> HashMap<&str, &str> {
    label_set
        .iter()
        .map(|(key, value)| (key.as_ref(), value.as_ref()))
        .collect()
}

/// Convert from list of `Key`s to prometheus' label format.
pub(crate) fn convert_labels(labels: &[Key]) -> Vec<&str> {
    labels.iter().map(|k| k.as_str()).collect()
}

/// Prometheus IntCounterHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct IntCounterHandle(prometheus::IntCounter);

impl api::Counter<i64, sdk::LabelSet> for prometheus::IntCounterVec {
    /// Prometheus' `CounterHandle`
    type Handle = IntCounterHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: i64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        IntCounterHandle(self.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for prometheus::IntCounterVec {
    /// Record a single counter measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.with(&convert_label_set(label_set))
            .inc_by(value.into_i64())
    }
}

impl api::InstrumentHandle for IntCounterHandle {
    /// Record a single counter measurement value for preset values
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.inc_by(value.into_i64())
    }
}

impl api::CounterHandle<i64> for IntCounterHandle {}

/// Prometheus CounterHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct CounterHandle(prometheus::Counter);

impl api::Counter<f64, sdk::LabelSet> for prometheus::CounterVec {
    type Handle = CounterHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: f64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        CounterHandle(self.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for prometheus::CounterVec {
    /// record a single counter measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.with(&convert_label_set(label_set))
            .inc_by(value.into_f64())
    }
}

impl api::InstrumentHandle for CounterHandle {
    /// record a single counter measurement value for precomputed labels
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.inc_by(value.into_f64())
    }
}

impl api::CounterHandle<f64> for CounterHandle {}

// GAUGE COMPAT

/// Prometheus IntGaugeHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct IntGaugeHandle(prometheus::IntGauge);

impl api::Gauge<i64, sdk::LabelSet> for prometheus::IntGaugeVec {
    type Handle = IntGaugeHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: i64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        IntGaugeHandle(self.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for prometheus::IntGaugeVec {
    /// record a single gauge measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.with(&convert_label_set(label_set))
            .set(value.into_i64())
    }
}

impl api::InstrumentHandle for IntGaugeHandle {
    /// record a single gauge measurement value for precomputed labels
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.set(value.into_i64())
    }
}

impl api::GaugeHandle<i64> for IntGaugeHandle {}

/// Prometheus GaugeHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct GaugeHandle(prometheus::Gauge);

impl api::Gauge<f64, sdk::LabelSet> for prometheus::GaugeVec {
    type Handle = GaugeHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: f64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        GaugeHandle(self.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for prometheus::GaugeVec {
    /// record a single gauge measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.with(&convert_label_set(label_set))
            .set(value.into_f64())
    }
}

impl api::InstrumentHandle for GaugeHandle {
    /// record a single gauge measurement value for precomputed labels
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.set(value.into_f64())
    }
}

impl api::GaugeHandle<f64> for GaugeHandle {}

// MEASURE COMPAT

/// Prometheus IntMeasureHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct IntMeasureHandle(prometheus::Histogram);

/// Prometheus Histograms do not have i64 variant, `IntMeasure` will convert i64 to float when it
/// records values.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct IntMeasure(prometheus::HistogramVec);

impl IntMeasure {
    pub(crate) fn new(histogram: prometheus::HistogramVec) -> Self {
        IntMeasure(histogram)
    }
}

impl api::Measure<i64, sdk::LabelSet> for IntMeasure {
    type Handle = IntMeasureHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: i64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        IntMeasureHandle(self.0.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for IntMeasure {
    /// record a single measure measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.0
            .with(&convert_label_set(label_set))
            .observe(value.into_i64() as f64)
    }
}

impl api::InstrumentHandle for IntMeasureHandle {
    /// record a single measure measurement value for precomputed labels
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.observe(value.into_i64() as f64)
    }
}

impl api::MeasureHandle<i64> for IntMeasureHandle {}

/// Prometheus MeasureHandle
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct MeasureHandle(prometheus::Histogram);

impl api::Measure<f64, sdk::LabelSet> for prometheus::HistogramVec {
    type Handle = MeasureHandle;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: f64) -> api::Measurement<sdk::LabelSet> {
        api::Measurement::new(Arc::new(self.clone()), api::MeasurementValue::from(value))
    }

    /// Creates a handle for this instrument.
    fn acquire_handle(&self, labels: &sdk::LabelSet) -> Self::Handle {
        MeasureHandle(self.with(&convert_label_set(labels)))
    }
}

impl api::Instrument<sdk::LabelSet> for prometheus::HistogramVec {
    /// record a single measure measurement value
    fn record_one(&self, value: api::MeasurementValue, label_set: &sdk::LabelSet) {
        self.with(&convert_label_set(label_set))
            .observe(value.into_f64())
    }
}

impl api::InstrumentHandle for MeasureHandle {
    /// record a single measure measurement value for precomputed labels
    fn record_one(&self, value: api::MeasurementValue) {
        self.0.observe(value.into_f64())
    }
}

impl api::MeasureHandle<f64> for MeasureHandle {}
