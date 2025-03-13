use opentelemetry::KeyValue;
use std::borrow::Cow;

pub trait MeasurementProcessor: Send + Sync + 'static {
    fn process<'a>(&self, attributes: Cow<'a, [KeyValue]>) -> Cow<'a, [KeyValue]>;
}

