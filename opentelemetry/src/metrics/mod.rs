//! # OpenTelemetry Metrics API

use std::hash::{Hash, Hasher};
use std::sync::Arc;

mod instruments;
mod meter;
pub(crate) mod noop;

use crate::{Array, KeyValue, Value};
pub use instruments::{
    counter::{Counter, ObservableCounter},
    gauge::{Gauge, ObservableGauge},
    histogram::Histogram,
    up_down_counter::{ObservableUpDownCounter, UpDownCounter},
    AsyncInstrument, AsyncInstrumentBuilder, Callback, HistogramBuilder, InstrumentBuilder,
    SyncInstrument,
};
pub use meter::{Meter, MeterProvider};

struct F64Hashable(f64);

impl PartialEq for F64Hashable {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F64Hashable {}

impl Hash for F64Hashable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Hash for KeyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        match &self.value {
            Value::F64(f) => F64Hashable(*f).hash(state),
            Value::Array(a) => match a {
                Array::Bool(b) => b.hash(state),
                Array::I64(i) => i.hash(state),
                Array::F64(f) => f.iter().for_each(|f| F64Hashable(*f).hash(state)),
                Array::String(s) => s.hash(state),
            },
            Value::Bool(b) => b.hash(state),
            Value::I64(i) => i.hash(state),
            Value::String(s) => s.hash(state),
        };
    }
}

impl Eq for KeyValue {}

/// SDK implemented trait for creating instruments
pub trait InstrumentProvider {
    /// creates an instrument for recording increasing values.
    fn u64_counter(&self, _builder: InstrumentBuilder<'_, Counter<u64>>) -> Counter<u64> {
        Counter::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording increasing values.
    fn f64_counter(&self, _builder: InstrumentBuilder<'_, Counter<f64>>) -> Counter<f64> {
        Counter::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording increasing values via callback.
    fn u64_observable_counter(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>,
    ) -> ObservableCounter<u64> {
        ObservableCounter::new()
    }

    /// creates an instrument for recording increasing values via callback.
    fn f64_observable_counter(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>,
    ) -> ObservableCounter<f64> {
        ObservableCounter::new()
    }

    /// creates an instrument for recording changes of a value.
    fn i64_up_down_counter(
        &self,
        _builder: InstrumentBuilder<'_, UpDownCounter<i64>>,
    ) -> UpDownCounter<i64> {
        UpDownCounter::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording changes of a value.
    fn f64_up_down_counter(
        &self,
        _builder: InstrumentBuilder<'_, UpDownCounter<f64>>,
    ) -> UpDownCounter<f64> {
        UpDownCounter::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording changes of a value.
    fn i64_observable_up_down_counter(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>,
    ) -> ObservableUpDownCounter<i64> {
        ObservableUpDownCounter::new()
    }

    /// creates an instrument for recording changes of a value via callback.
    fn f64_observable_up_down_counter(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>,
    ) -> ObservableUpDownCounter<f64> {
        ObservableUpDownCounter::new()
    }

    /// creates an instrument for recording independent values.
    fn u64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<u64>>) -> Gauge<u64> {
        Gauge::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording independent values.
    fn f64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<f64>>) -> Gauge<f64> {
        Gauge::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording independent values.
    fn i64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<i64>>) -> Gauge<i64> {
        Gauge::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording the current value via callback.
    fn u64_observable_gauge(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>,
    ) -> ObservableGauge<u64> {
        ObservableGauge::new()
    }

    /// creates an instrument for recording the current value via callback.
    fn i64_observable_gauge(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>,
    ) -> ObservableGauge<i64> {
        ObservableGauge::new()
    }

    /// creates an instrument for recording the current value via callback.
    fn f64_observable_gauge(
        &self,
        _builder: AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>,
    ) -> ObservableGauge<f64> {
        ObservableGauge::new()
    }

    /// creates an instrument for recording a distribution of values.
    fn f64_histogram(&self, _builder: HistogramBuilder<'_, Histogram<f64>>) -> Histogram<f64> {
        Histogram::new(Arc::new(noop::NoopSyncInstrument::new()))
    }

    /// creates an instrument for recording a distribution of values.
    fn u64_histogram(&self, _builder: HistogramBuilder<'_, Histogram<u64>>) -> Histogram<u64> {
        Histogram::new(Arc::new(noop::NoopSyncInstrument::new()))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::KeyValue;
    use std::collections::hash_map::DefaultHasher;
    use std::f64;
    use std::hash::{Hash, Hasher};

    #[test]
    fn kv_float_equality() {
        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.0);
        assert_eq!(kv1, kv2);

        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.01);
        assert_ne!(kv1, kv2);

        let kv1 = KeyValue::new("key", f64::NAN);
        let kv2 = KeyValue::new("key", f64::NAN);
        assert_ne!(kv1, kv2, "NAN is not equal to itself");

        for float_val in [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
        ]
        .iter()
        {
            let kv1 = KeyValue::new("key", *float_val);
            let kv2 = KeyValue::new("key", *float_val);
            assert_eq!(kv1, kv2);
        }

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(kv1, kv2);
        }
    }

    #[test]
    fn kv_float_hash() {
        for float_val in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
        ]
        .iter()
        {
            let kv1 = KeyValue::new("key", *float_val);
            let kv2 = KeyValue::new("key", *float_val);
            assert_eq!(hash_helper(&kv1), hash_helper(&kv2));
        }

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(hash_helper(&kv1), hash_helper(&kv2));
        }
    }

    fn hash_helper<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}
