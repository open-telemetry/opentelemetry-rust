//! # OpenTelemetry Metrics API

use std::any::Any;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::result;
use std::sync::PoisonError;
use std::{borrow::Cow, sync::Arc};
use thiserror::Error;

mod instruments;
mod meter;
pub mod noop;

use crate::{Array, ExportError, KeyValue, Value};
pub use instruments::{
    counter::{Counter, ObservableCounter, SyncCounter},
    gauge::{Gauge, ObservableGauge, SyncGauge},
    histogram::{Histogram, SyncHistogram},
    up_down_counter::{ObservableUpDownCounter, SyncUpDownCounter, UpDownCounter},
    AsyncInstrument, AsyncInstrumentBuilder, Callback, InstrumentBuilder,
};
pub use meter::{CallbackRegistration, Meter, MeterProvider, Observer};

/// A specialized `Result` type for metric operations.
pub type Result<T> = result::Result<T, MetricsError>;

/// Errors returned by the metrics API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MetricsError {
    /// Other errors not covered by specific cases.
    #[error("Metrics error: {0}")]
    Other(String),
    /// Invalid configuration
    #[error("Config error {0}")]
    Config(String),
    /// Fail to export metrics
    #[error("Metrics exporter {} failed with {0}", .0.exporter_name())]
    ExportErr(Box<dyn ExportError>),
    /// Invalid instrument configuration such invalid instrument name, invalid instrument description, invalid instrument unit, etc.
    /// See [spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#general-characteristics)
    /// for full list of requirements.
    #[error("Invalid instrument configuration: {0}")]
    InvalidInstrumentConfiguration(&'static str),
}

impl<T: ExportError> From<T> for MetricsError {
    fn from(err: T) -> Self {
        MetricsError::ExportErr(Box::new(err))
    }
}

impl<T> From<PoisonError<T>> for MetricsError {
    fn from(err: PoisonError<T>) -> Self {
        MetricsError::Other(err.to_string())
    }
}

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

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ordering is based on the key only.
impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl Eq for KeyValue {}

/// SDK implemented trait for creating instruments
pub trait InstrumentProvider {
    /// creates an instrument for recording increasing values.
    fn u64_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Counter<u64>> {
        Ok(Counter::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording increasing values.
    fn f64_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Counter<f64>> {
        Ok(Counter::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording increasing values via callback.
    fn u64_observable_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<u64>>,
    ) -> Result<ObservableCounter<u64>> {
        Ok(ObservableCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording increasing values via callback.
    fn f64_observable_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<f64>>,
    ) -> Result<ObservableCounter<f64>> {
        Ok(ObservableCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording changes of a value.
    fn i64_up_down_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<UpDownCounter<i64>> {
        Ok(UpDownCounter::new(
            Arc::new(noop::NoopSyncInstrument::new()),
        ))
    }

    /// creates an instrument for recording changes of a value.
    fn f64_up_down_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<UpDownCounter<f64>> {
        Ok(UpDownCounter::new(
            Arc::new(noop::NoopSyncInstrument::new()),
        ))
    }

    /// creates an instrument for recording changes of a value.
    fn i64_observable_up_down_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<i64>>,
    ) -> Result<ObservableUpDownCounter<i64>> {
        Ok(ObservableUpDownCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording changes of a value via callback.
    fn f64_observable_up_down_counter(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<f64>>,
    ) -> Result<ObservableUpDownCounter<f64>> {
        Ok(ObservableUpDownCounter::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording independent values.
    fn u64_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Gauge<u64>> {
        Ok(Gauge::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording independent values.
    fn f64_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Gauge<f64>> {
        Ok(Gauge::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording independent values.
    fn i64_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Gauge<i64>> {
        Ok(Gauge::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording the current value via callback.
    fn u64_observable_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<u64>>,
    ) -> Result<ObservableGauge<u64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording the current value via callback.
    fn i64_observable_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<i64>>,
    ) -> Result<ObservableGauge<i64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording the current value via callback.
    fn f64_observable_gauge(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
        _callback: Vec<Callback<f64>>,
    ) -> Result<ObservableGauge<f64>> {
        Ok(ObservableGauge::new(Arc::new(
            noop::NoopAsyncInstrument::new(),
        )))
    }

    /// creates an instrument for recording a distribution of values.
    fn f64_histogram(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Histogram<f64>> {
        Ok(Histogram::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// creates an instrument for recording a distribution of values.
    fn u64_histogram(
        &self,
        _name: Cow<'static, str>,
        _description: Option<Cow<'static, str>>,
        _unit: Option<Cow<'static, str>>,
    ) -> Result<Histogram<u64>> {
        Ok(Histogram::new(Arc::new(noop::NoopSyncInstrument::new())))
    }

    /// Captures the function that will be called during data collection.
    ///
    /// It is only valid to call `observe` within the scope of the passed function.
    fn register_callback(
        &self,
        instruments: &[Arc<dyn Any>],
        callbacks: Box<MultiInstrumentCallback>,
    ) -> Result<Box<dyn CallbackRegistration>>;
}

type MultiInstrumentCallback = dyn Fn(&dyn Observer) + Send + Sync;

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

    #[test]
    fn kv_float_order() {
        // TODO: Extend this test to all value types, not just F64
        let float_vals = [
            0.0,
            1.0,
            -1.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MIN,
            f64::MAX,
        ];

        for v in float_vals {
            let kv1 = KeyValue::new("a", v);
            let kv2 = KeyValue::new("b", v);
            assert!(kv1 < kv2, "Order is solely based on key!");
        }
    }

    fn hash_helper<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}
