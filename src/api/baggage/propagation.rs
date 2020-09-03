use super::Baggage;
use crate::api::context::propagation::text_propagator::FieldIter;
use crate::api::{self, Context, KeyValue};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use std::iter;

static BAGGAGE_HEADER: &str = "otcorrelations";
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b';').add(b',').add(b'=');

lazy_static::lazy_static! {
    static ref DEFAULT_BAGGAGE: Baggage = Baggage::default();
    static ref BAGGAGE_FIELDS: [String; 1] = [BAGGAGE_HEADER.to_string()];
}

/// Propagates name/value pairs in [W3C Baggage] format.
///
/// [W3C Baggage]: https://w3c.github.io/baggage
#[derive(Debug, Default)]
pub struct BaggagePropagator {
    _private: (),
}

impl BaggagePropagator {
    /// Construct a new baggage propagator.
    pub fn new() -> Self {
        BaggagePropagator { _private: () }
    }
}

impl api::TextMapFormat for BaggagePropagator {
    /// Encodes the values of the `Context` and injects them into the provided `Injector`.
    fn inject_context(&self, cx: &Context, injector: &mut dyn api::Injector) {
        let baggage = cx.baggage();
        if !baggage.is_empty() {
            let header_value = baggage
                .iter()
                .map(|(name, value)| {
                    utf8_percent_encode(name.as_str().trim(), FRAGMENT)
                        .chain(iter::once("="))
                        .chain(utf8_percent_encode(String::from(value).trim(), FRAGMENT))
                        .collect()
                })
                .collect::<Vec<String>>()
                .join(",");
            injector.set(BAGGAGE_HEADER, header_value);
        }
    }

    /// Extracts a `Context` with baggage values from a `Extractor`.
    fn extract_with_context(&self, cx: &Context, extractor: &dyn api::Extractor) -> Context {
        if let Some(header_value) = extractor.get(BAGGAGE_HEADER) {
            let baggage = header_value.split(',').flat_map(|context_value| {
                if let Some((name_and_value, props)) = context_value
                    .split(';')
                    .collect::<Vec<&str>>()
                    .split_first()
                {
                    let mut iter = name_and_value.split('=');
                    if let (Some(name), Some(value)) = (iter.next(), iter.next()) {
                        let name = percent_decode_str(name).decode_utf8().map_err(|_| ())?;
                        let value = percent_decode_str(value).decode_utf8().map_err(|_| ())?;

                        // TODO: handle props from https://w3c.github.io/baggage/
                        // for now just append to value
                        let decoded_props = props
                            .iter()
                            .flat_map(|prop| percent_decode_str(prop).decode_utf8())
                            .map(|prop| format!(";{}", prop.as_ref().trim()))
                            .collect::<String>();

                        Ok(KeyValue::new(
                            name.trim().to_owned(),
                            value.trim().to_string() + decoded_props.as_str(),
                        ))
                    } else {
                        // Invalid name / value format
                        Err(())
                    }
                } else {
                    // Invalid baggage value format
                    Err(())
                }
            });
            cx.with_baggage(baggage)
        } else {
            cx.clone()
        }
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(BAGGAGE_FIELDS.as_ref())
    }
}

struct BaggageWrapper(Baggage);

/// Methods for sorting and retrieving baggage data in a context.
pub trait BaggageExt {
    /// Returns a clone of the current context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, BaggageExt, KeyValue, Value};
    ///
    /// let cx = Context::current_with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn current_with_baggage<T: IntoIterator<Item = KeyValue>>(baggage: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, BaggageExt, KeyValue, Value};
    ///
    /// let some_context = Context::current();
    /// let cx = some_context.with_baggage(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn with_baggage<T: IntoIterator<Item = KeyValue>>(&self, baggage: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, BaggageExt, KeyValue, Value};
    ///
    /// let cx = Context::current().with_cleared_baggage();
    ///
    /// assert_eq!(cx.baggage().len(), 0);
    /// ```
    fn with_cleared_baggage(&self) -> Self;

    /// Returns a reference to this context's baggage, or the default
    /// empty baggage if none has been set.
    fn baggage(&self) -> &Baggage;
}

impl BaggageExt for Context {
    fn current_with_baggage<T: IntoIterator<Item = KeyValue>>(kvs: T) -> Self {
        Context::current().with_baggage(kvs)
    }

    fn with_baggage<T: IntoIterator<Item = KeyValue>>(&self, kvs: T) -> Self {
        let merged = self
            .baggage()
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .chain(kvs.into_iter())
            .collect();

        self.with_value(BaggageWrapper(merged))
    }

    fn with_cleared_baggage(&self) -> Self {
        self.with_value(BaggageWrapper(Baggage::new()))
    }

    fn baggage(&self) -> &Baggage {
        self.get::<BaggageWrapper>()
            .map(|baggage| &baggage.0)
            .unwrap_or_else(|| &DEFAULT_BAGGAGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::TextMapFormat;
    use crate::api::{Key, Value};
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn valid_extract_data() -> Vec<(&'static str, HashMap<Key, Value>)> {
        vec![
            // "valid w3cHeader"
            ("key1=val1,key2=val2", vec![(Key::new("key1"), Value::from("val1")), (Key::new("key2"), Value::from("val2"))].into_iter().collect()),
            // "valid w3cHeader with spaces"
            ("key1 =   val1,  key2 =val2   ", vec![(Key::new("key1"), Value::from("val1")), (Key::new("key2"), Value::from("val2"))].into_iter().collect()),
            // "valid w3cHeader with properties"
            ("key1=val1,key2=val2;prop=1", vec![(Key::new("key1"), Value::from("val1")), (Key::new("key2"), Value::from("val2;prop=1"))].into_iter().collect()),
            // "valid header with url-escaped comma"
            ("key1=val1,key2=val2%2Cval3", vec![(Key::new("key1"), Value::from("val1")), (Key::new("key2"), Value::from("val2,val3"))].into_iter().collect()),
            // "valid header with an invalid header"
            ("key1=val1,key2=val2,a,val3", vec![(Key::new("key1"), Value::from("val1")), (Key::new("key2"), Value::from("val2"))].into_iter().collect()),
            // "valid header with no value"
            ("key1=,key2=val2", vec![(Key::new("key1"), Value::from("")), (Key::new("key2"), Value::from("val2"))].into_iter().collect()),
        ]
    }

    #[rustfmt::skip]
    fn valid_inject_data() -> Vec<(Vec<KeyValue>, Vec<&'static str>)> {
        vec![
            // "two simple values"
            (vec![KeyValue::new("key1", "val1"), KeyValue::new("key2", "val2")], vec!["key1=val1", "key2=val2"]),
            // "two values with escaped chars"
            (vec![KeyValue::new("key1", "val1,val2"), KeyValue::new("key2", "val3=4")], vec!["key1=val1%2Cval2", "key2=val3%3D4"]),
            // "values of non-string non-array types"
            (
                vec![
                    KeyValue::new("key1", true),
                    KeyValue::new("key2", Value::I64(123)),
                    KeyValue::new("key3", Value::U64(123)),
                    KeyValue::new("key4", Value::F64(123.567)),
                ],
                vec![
                    "key1=true",
                    "key2=123",
                    "key3=123",
                    "key4=123.567",
                ],
            ),
            // "values of array types"
            (
                vec![
                    KeyValue::new("key1", Value::Array(vec![Value::Bool(true), Value::Bool(false)])),
                    KeyValue::new("key2", Value::Array(vec![Value::I64(123), Value::I64(456)])),
                    KeyValue::new("key3", Value::Array(vec![Value::String("val1".to_string()), Value::String("val2".to_string())])),
                    KeyValue::new("key4", Value::Array(vec![Value::Bytes(vec![118, 97, 108, 49]), Value::Bytes(vec![118, 97, 108, 50])])),
                ],
                vec![
                    "key1=[true%2Cfalse]",
                    "key2=[123%2C456]",
                    "key3=[%22val1%22%2C%22val2%22]",
                    "key4=[%22val1%22%2C%22val2%22]",
                ],
            )
        ]
    }

    #[test]
    fn extract_baggage() {
        let propagator = BaggagePropagator::new();

        for (header_value, kvs) in valid_extract_data() {
            let mut extractor: HashMap<String, String> = HashMap::new();
            extractor.insert(
                BAGGAGE_HEADER.to_string(),
                header_value.to_string(),
            );
            let context = propagator.extract(&extractor);
            let baggage = context.baggage();

            assert_eq!(kvs.len(), baggage.len());
            for (key, value) in baggage {
                assert_eq!(Some(value), kvs.get(key))
            }
        }
    }

    #[test]
    fn inject_baggage() {
        let propagator = BaggagePropagator::new();

        for (kvs, header_parts) in valid_inject_data() {
            let mut injector = HashMap::new();
            let cx = Context::current_with_baggage(kvs);
            propagator.inject_context(&cx, &mut injector);
            let header_value = injector.get(BAGGAGE_HEADER).unwrap();

            assert_eq!(header_parts.join(",").len(), header_value.len(),);
            for header_part in &header_parts {
                assert!(header_value.contains(header_part),)
            }
        }
    }
}
