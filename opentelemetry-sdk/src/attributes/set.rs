use crate::Resource;
use opentelemetry::attributes::AttributeSet;
use opentelemetry::KeyValue;

impl From<&Resource> for AttributeSet {
    fn from(values: &Resource) -> Self {
        let key_values = values
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect::<Vec<_>>();

        AttributeSet::from(key_values.as_slice())
    }
}
