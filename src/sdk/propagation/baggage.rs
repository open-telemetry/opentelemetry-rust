//! # OpenTelemetry Baggage API
//!
//! Baggage is used to annotate telemetry, adding context and
//! information to metrics, traces, and logs. It is an abstract data type
//! represented by a set of name/value pairs describing user-defined properties.
//! Each name in a [`Baggage`] is associated with exactly one value.
//! `Baggage`s are serialized according to the editor's draft of
//! the [W3C Baggage] specification.
//!
//! [`CorrelationContext`]: struct.Baggage.html
//! [W3C Baggage]: https://w3c.github.io/baggage/
//!
//! # Examples
//!
//! ```
//! use opentelemetry::api::{BaggageExt, Key, propagation::TextMapPropagator};
//! use opentelemetry::sdk::propagation::BaggagePropagator;
//! use std::collections::HashMap;
//!
//! // Example baggage value passed in externally via http headers
//! let mut headers = HashMap::new();
//! headers.insert("baggage".to_string(), "user_id=1".to_string());
//!
//! let propagator = BaggagePropagator::new();
//! // can extract from any type that impls `Extractor`, usually an HTTP header map
//! let cx = propagator.extract(&headers);
//!
//! // Iterate over extracted name / value pairs
//! for (name, value) in cx.baggage() {
//!     // ...
//! }
//!
//! // Add new baggage
//! let cx_with_additions = cx.with_baggage(vec![Key::new("server_id").u64(42)]);
//!
//! // Inject aggage into http request
//! propagator.inject_context(&cx_with_additions, &mut headers);
//!
//! let header_value = headers.get("baggage").expect("header is injected");
//! assert!(header_value.contains("user_id=1"), "still contains previous name / value");
//! assert!(header_value.contains("server_id=42"), "contains new name / value pair");
//! ```
use crate::api::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    BaggageExt, Context, KeyValue,
};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use std::iter;

static BAGGAGE_HEADER: &str = "baggage";
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b';').add(b',').add(b'=');

lazy_static::lazy_static! {
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

impl TextMapPropagator for BaggagePropagator {
    /// Encodes the values of the `Context` and injects them into the provided `Injector`.
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
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
    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{propagation::TextMapPropagator, Key, Value};
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
            extractor.insert(BAGGAGE_HEADER.to_string(), header_value.to_string());
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
