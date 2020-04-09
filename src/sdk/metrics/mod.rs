//! # OpenTelemetry Metrics SDK
//!
//! The metrics SDK supports producing diagnostic measurements
//! using three basic kinds of `Instrument`s. "Metrics" are the thing being
//! produced--mathematical, statistical summaries of certain observable
//! behavior in the program. `Instrument`s are the devices used by the
//! program to record observations about their behavior. Therefore, we use
//! "metric instrument" to refer to a program object, allocated through the
//! `Meter` struct, used for recording metrics. There are three distinct
//! instruments in the Metrics API, commonly known as `Counter`s, `Gauge`s,
//! and `Measure`s.
use crate::api;
use crate::exporter::metrics::prometheus;
use std::borrow::Cow;
use std::collections::HashMap;

/// Collection of label key and value types.
pub type LabelSet = HashMap<Cow<'static, str>, Cow<'static, str>>;
impl api::LabelSet for LabelSet {}

/// `Meter` implementation to create manage metric instruments and record
/// batch measurements
#[allow(missing_debug_implementations)]
pub struct Meter {
    registry: &'static prometheus::Registry,
    component: &'static str,
}

impl Meter {
    /// Create a new `Meter` instance with a component name and empty registry.
    pub fn new(component: &'static str) -> Self {
        Meter {
            registry: prometheus::default_registry(),
            component,
        }
    }

    /// Build prometheus `Opts` from `name` and `description`.
    fn build_opts(
        &self,
        mut name: String,
        unit: api::Unit,
        description: String,
    ) -> prometheus::Opts {
        if !unit.as_str().is_empty() {
            name.push_str(&format!("_{}", unit.as_str()));
        }
        // Prometheus cannot have empty help strings
        let help = if !description.is_empty() {
            description
        } else {
            format!("{} metric", name)
        };
        prometheus::Opts::new(name, help).namespace(self.component)
    }
}

impl api::Meter for Meter {
    /// The label set used by this `Meter`.
    type LabelSet = LabelSet;
    /// This implementation of `api::Meter` produces `prometheus::IntCounterVec;` instances.
    type I64Counter = prometheus::IntCounterVec;
    /// This implementation of `api::Meter` produces `prometheus::CounterVec;` instances.
    type F64Counter = prometheus::CounterVec;
    /// This implementation of `api::Meter` produces `prometheus::IntGaugeVec;` instances.
    type I64Gauge = prometheus::IntGaugeVec;
    /// This implementation of `api::Meter` produces `prometheus::GaugeVec;` instances.
    type F64Gauge = prometheus::GaugeVec;
    /// This implementation of `api::Meter` produces `prometheus::IntMeasure;` instances.
    type I64Measure = prometheus::IntMeasure;
    /// This implementation of `api::Meter` produces `prometheus::HistogramVec;` instances.
    type F64Measure = prometheus::HistogramVec;

    /// Builds a `LabelSet` from `KeyValue`s.
    fn labels(&self, key_values: Vec<api::KeyValue>) -> Self::LabelSet {
        let mut label_set: Self::LabelSet = Default::default();

        for api::KeyValue { key, value } in key_values.into_iter() {
            label_set.insert(Cow::Owned(key.into()), Cow::Owned(value.into()));
        }

        label_set
    }

    /// Creates a new `i64` counter with a given name and customized with passed options.
    fn new_i64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: api::MetricOptions,
    ) -> Self::I64Counter {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let counter_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let counter = prometheus::IntCounterVec::new(counter_opts, &labels).unwrap();
        self.registry.register(Box::new(counter.clone())).unwrap();

        counter
    }

    /// Creates a new `f64` counter with a given name and customized with passed options.
    fn new_f64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: api::MetricOptions,
    ) -> Self::F64Counter {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let counter_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let counter = prometheus::CounterVec::new(counter_opts, &labels).unwrap();
        self.registry.register(Box::new(counter.clone())).unwrap();

        counter
    }

    /// Creates a new `i64` gauge with a given name and customized with passed options.
    fn new_i64_gauge<S: Into<String>>(&self, name: S, opts: api::MetricOptions) -> Self::I64Gauge {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let gauge_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let gauge = prometheus::IntGaugeVec::new(gauge_opts, &labels).unwrap();
        self.registry.register(Box::new(gauge.clone())).unwrap();

        gauge
    }

    /// Creates a new `f64` gauge with a given name and customized with passed options.
    fn new_f64_gauge<S: Into<String>>(&self, name: S, opts: api::MetricOptions) -> Self::F64Gauge {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let gauge_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let gauge = prometheus::GaugeVec::new(gauge_opts, &labels).unwrap();
        self.registry.register(Box::new(gauge.clone())).unwrap();

        gauge
    }

    /// Creates a new `i64` measure with a given name and customized with passed options.
    fn new_i64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: api::MetricOptions,
    ) -> Self::I64Measure {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let common_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let histogram_opts = prometheus::HistogramOpts::from(common_opts);
        let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
        self.registry.register(Box::new(histogram.clone())).unwrap();

        prometheus::IntMeasure::new(histogram)
    }

    /// Creates a new `f64` measure with a given name and customized with passed options.
    fn new_f64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: api::MetricOptions,
    ) -> Self::F64Measure {
        let api::MetricOptions {
            description,
            unit,
            keys,
            alternate: _alternative,
        } = opts;
        let common_opts = self.build_opts(name.into(), unit, description);
        let labels = prometheus::convert_labels(&keys);
        let histogram_opts = prometheus::HistogramOpts::from(common_opts);
        let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
        self.registry.register(Box::new(histogram.clone())).unwrap();

        histogram
    }

    /// Records a batch of measurements.
    fn record_batch<M: IntoIterator<Item = api::Measurement<Self::LabelSet>>>(
        &self,
        label_set: &Self::LabelSet,
        measurements: M,
    ) {
        for measure in measurements.into_iter() {
            let instrument = measure.instrument();
            instrument.record_one(measure.into_value(), &label_set);
        }
    }
}
