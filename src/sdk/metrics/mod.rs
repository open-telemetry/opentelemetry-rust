use crate::api::metrics;

mod prometheus_metrics;

pub struct Meter {
    registry: &'static prometheus::Registry,
    component: &'static str,
}

impl Meter {
    pub fn new(component: &'static str) -> Self {
        Meter {
            registry: prometheus::default_registry(),
            component,
        }
    }

    fn build_opts(&self, name: String, description: String) -> prometheus::Opts {
        // Prometheus cannot have empty help strings
        let help = if !description.is_empty() {
            description
        } else {
            format!("{} metric", name)
        };
        prometheus::Opts::new(name, help).namespace(format!("{}_", self.component))
    }
}

impl metrics::Meter for Meter {
    type LabelSet = prometheus_metrics::LabelSet;
    type I64Counter = prometheus::IntCounterVec;
    type F64Counter = prometheus::CounterVec;
    type I64Gauge = prometheus::IntGaugeVec;
    type F64Gauge = prometheus::GaugeVec;
    type I64Measure = prometheus_metrics::IntMeasure;
    type F64Measure = prometheus::HistogramVec;

    fn labels(&self, key_values: Vec<crate::KeyValue>) -> Self::LabelSet {
        let mut label_set: Self::LabelSet = Default::default();

        for crate::KeyValue { key, value } in key_values.into_iter() {
            label_set.insert(key.into(), value.into());
        }

        label_set
    }

    fn new_i64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: metrics::Options,
    ) -> Self::I64Counter {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let counter_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let counter = prometheus::IntCounterVec::new(counter_opts, &labels).unwrap();
        self.registry.register(Box::new(counter.clone())).unwrap();

        counter
    }

    fn new_f64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: metrics::Options,
    ) -> Self::F64Counter {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let counter_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let counter = prometheus::CounterVec::new(counter_opts, &labels).unwrap();
        self.registry.register(Box::new(counter.clone())).unwrap();

        counter
    }

    fn new_i64_gauge<S: Into<String>>(&self, name: S, opts: metrics::Options) -> Self::I64Gauge {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let gauge_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let gauge = prometheus::IntGaugeVec::new(gauge_opts, &labels).unwrap();
        self.registry.register(Box::new(gauge.clone())).unwrap();

        gauge
    }

    fn new_f64_gauge<S: Into<String>>(&self, name: S, opts: metrics::Options) -> Self::F64Gauge {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let gauge_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let gauge = prometheus::GaugeVec::new(gauge_opts, &labels).unwrap();
        self.registry.register(Box::new(gauge.clone())).unwrap();

        gauge
    }

    fn new_i64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: metrics::Options,
    ) -> Self::I64Measure {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let common_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let histogram_opts = prometheus::HistogramOpts::from(common_opts);
        let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
        self.registry.register(Box::new(histogram.clone())).unwrap();

        prometheus_metrics::IntMeasure::new(histogram)
    }

    fn new_f64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: metrics::Options,
    ) -> Self::F64Measure {
        let metrics::Options {
            description,
            unit: _unit,
            keys,
            alternate: _alternative,
        } = opts;
        let common_opts = self.build_opts(name.into(), description);
        let labels = prometheus_metrics::convert_labels(&keys);
        let histogram_opts = prometheus::HistogramOpts::from(common_opts);
        let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
        self.registry.register(Box::new(histogram.clone())).unwrap();

        histogram
    }

    fn record_batch<M: IntoIterator<Item = metrics::Measurement<Self::LabelSet>>>(
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
