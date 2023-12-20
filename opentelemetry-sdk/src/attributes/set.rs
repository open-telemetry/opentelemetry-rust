use crate::Resource;
use opentelemetry::attributes::AttributeSet;
use opentelemetry::KeyValue;

impl From<&Resource> for AttributeSet {
    fn from(values: &Resource) -> Self {
        let key_values = values
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect::<Vec<_>>();

        AttributeSet::from(&key_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{Key, Value};

    #[test]
    fn can_create_attribute_set_from_resource() {
        let resource = Resource::new([KeyValue::new("key1", "value1"), KeyValue::new("key2", 3)]);

        let set = AttributeSet::from(&resource);
        let mut kvs = set.iter().collect::<Vec<(&Key, &Value)>>();

        assert_eq!(kvs.len(), 2, "Incorrect number of attributes");

        kvs.sort_by(|kv1, kv2| kv1.0.cmp(kv2.0));
        assert_eq!(kvs[0].0, &Key::from("key1"), "Unexpected first key");
        assert_eq!(
            kvs[0].1,
            &Value::String("value1".into()),
            "Unexpected first value"
        );
        assert_eq!(kvs[1].0, &Key::from("key2"), "Unexpected second key");
        assert_eq!(kvs[1].1, &Value::I64(3), "Unexpected second value");
    }
}
