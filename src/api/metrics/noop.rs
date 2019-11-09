//! # No-op OpenTelemetry Metrics Implementation
//!
//! This implementation is returned as the global Meter if no `Meter`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api;
use std::marker;
use std::sync::Arc;

/// A no-op instance of a `Meter`.
#[derive(Debug)]
pub struct NoopMeter {}

impl api::Meter for NoopMeter {
    type LabelSet = NoopLabelSet;
    type I64Counter = NoopCounter<i64>;
    type F64Counter = NoopCounter<f64>;
    type I64Gauge = NoopGauge<i64>;
    type F64Gauge = NoopGauge<f64>;
    type I64Measure = NoopMeasure<i64>;
    type F64Measure = NoopMeasure<f64>;

    /// Returns a no-op `NoopLabelSet`.
    fn labels(&self, _key_values: Vec<api::KeyValue>) -> Self::LabelSet {
        NoopLabelSet {}
    }

    /// Returns a no-op `I64Counter` instance.
    fn new_i64_counter<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::I64Counter {
        NoopCounter {
            _marker: marker::PhantomData,
        }
    }

    /// Returns a no-op `F64Counter` instance.
    fn new_f64_counter<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::F64Counter {
        NoopCounter {
            _marker: marker::PhantomData,
        }
    }

    /// Returns a no-op `I64Gauge` instance.
    fn new_i64_gauge<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::I64Gauge {
        NoopGauge {
            _marker: marker::PhantomData,
        }
    }

    /// Returns a no-op `F64Gauge` instance.
    fn new_f64_gauge<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::F64Gauge {
        NoopGauge {
            _marker: marker::PhantomData,
        }
    }

    /// Returns a no-op `I64Measure` instance.
    fn new_i64_measure<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::I64Measure {
        NoopMeasure {
            _marker: marker::PhantomData,
        }
    }

    /// Returns a no-op `F64Measure` instance.
    fn new_f64_measure<S: Into<String>>(
        &self,
        _name: S,
        _opts: api::MetricOptions,
    ) -> Self::F64Measure {
        NoopMeasure {
            _marker: marker::PhantomData,
        }
    }

    /// Ignores batch recordings
    fn record_batch<M: IntoIterator<Item = api::Measurement<NoopLabelSet>>>(
        &self,
        _label_set: &NoopLabelSet,
        _measurements: M,
    ) {
        // Ignored
    }
}

/// A no-op instance of `LabelSet`.
#[derive(Debug)]
pub struct NoopLabelSet {}

impl api::LabelSet for NoopLabelSet {}

/// A no-op instance of all metric `InstrumentHandler`
#[derive(Debug)]
pub struct NoopHandle<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> api::Instrument<NoopLabelSet> for NoopHandle<T> {
    fn record_one(&self, _value: api::MeasurementValue, _label_set: &NoopLabelSet) {
        // Ignored
    }
}

impl<T> api::CounterHandle<T> for NoopHandle<T> where T: Into<api::MeasurementValue> {}

impl<T> api::GaugeHandle<T> for NoopHandle<T> where T: Into<api::MeasurementValue> {}

impl<T> api::MeasureHandle<T> for NoopHandle<T> where T: Into<api::MeasurementValue> {}

/// A no-op instance of a `Counter`.
#[derive(Debug)]
pub struct NoopCounter<T> {
    _marker: marker::PhantomData<T>,
}

impl<T: Into<api::MeasurementValue> + 'static> api::Counter<T, NoopLabelSet> for NoopCounter<T> {
    type Handle = NoopHandle<T>;

    /// Returns a no-op `Measurement`.
    fn measurement(&self, value: T) -> api::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        api::Measurement {
            instrument: Arc::new(handle),
            value: value.into(),
        }
    }

    /// Returns a `NoopHandle`
    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> api::Instrument<NoopLabelSet> for NoopCounter<T> {
    /// Ignores all recorded measurement values.
    fn record_one(&self, _value: api::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}

/// A no-op instance of a `Gauge`.
#[derive(Debug)]
pub struct NoopGauge<T> {
    _marker: marker::PhantomData<T>,
}

impl api::Gauge<i64, NoopLabelSet> for NoopGauge<i64> {
    type Handle = NoopHandle<i64>;

    /// Returns a no-op `Measurement`.
    fn measurement(&self, value: i64) -> api::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        api::Measurement {
            instrument: Arc::new(handle),
            value: api::MeasurementValue::from(value),
        }
    }

    /// Returns a `NoopHandle`
    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl api::Gauge<f64, NoopLabelSet> for NoopGauge<f64> {
    type Handle = NoopHandle<f64>;

    /// Returns a no-op `Measurement`.
    fn measurement(&self, value: f64) -> api::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});
        api::Measurement {
            instrument: Arc::new(handle),
            value: api::MeasurementValue::from(value),
        }
    }

    /// Returns a `NoopHandle`
    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> api::InstrumentHandle for NoopHandle<T> {
    /// Ignores all measurement values.
    fn record_one(&self, _value: api::MeasurementValue) {
        // Ignored
    }
}

impl<T> api::Instrument<NoopLabelSet> for NoopGauge<T> {
    /// Ignores all measurement values and labels.
    fn record_one(&self, _value: api::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}

/// A no-op instance of a `Measure`.
#[derive(Debug)]
pub struct NoopMeasure<T> {
    _marker: marker::PhantomData<T>,
}

impl api::Measure<i64, NoopLabelSet> for NoopMeasure<i64> {
    type Handle = NoopHandle<i64>;

    /// Returns a no-op `Measurement`.
    fn measurement(&self, value: i64) -> api::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});

        api::Measurement::new(Arc::new(handle), api::MeasurementValue::from(value))
    }

    /// Returns a `NoopHandle`
    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl api::Measure<f64, NoopLabelSet> for NoopMeasure<f64> {
    type Handle = NoopHandle<f64>;

    /// Returns a no-op `Measurement`.
    fn measurement(&self, value: f64) -> api::Measurement<NoopLabelSet> {
        let handle = self.acquire_handle(&NoopLabelSet {});

        api::Measurement::new(Arc::new(handle), api::MeasurementValue::from(value))
    }

    /// Returns a `NoopHandle`
    fn acquire_handle(&self, _labels: &NoopLabelSet) -> Self::Handle {
        NoopHandle {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> api::Instrument<NoopLabelSet> for NoopMeasure<T> {
    /// Ignores all measurement values and labels.
    fn record_one(&self, _value: api::MeasurementValue, _labels: &NoopLabelSet) {
        // Ignored
    }
}
