use super::CorrelationContext;
use crate::api::{self, Context, KeyValue};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use std::iter;

static CORRELATION_CONTEXT_HEADER: &str = "Correlation-Context";
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b';').add(b',').add(b'=');

lazy_static::lazy_static! {
    static ref DEFAULT_CORRELATION_CONTEXT: CorrelationContext = CorrelationContext::default();
}

/// Propagates name/value pairs in [W3C Correlation Context] format.
///
/// [W3C Correlation Context]: https://w3c.github.io/correlation-context/
#[derive(Debug, Default)]
pub struct CorrelationContextPropagator {
    _private: (),
}

impl CorrelationContextPropagator {
    /// Construct a new correlation context provider.
    pub fn new() -> Self {
        CorrelationContextPropagator { _private: () }
    }
}

impl api::HttpTextFormat for CorrelationContextPropagator {
    /// Encodes the values of the `Context` and injects them into the provided `Carrier`.
    fn inject_context(&self, cx: &Context, carrier: &mut dyn api::Carrier) {
        let correlation_cx = cx.correlation_context();
        if !correlation_cx.is_empty() {
            let header_value = correlation_cx
                .iter()
                .map(|(name, value)| {
                    utf8_percent_encode(name.as_str().trim(), FRAGMENT)
                        .chain(iter::once("="))
                        .chain(utf8_percent_encode(String::from(value).trim(), FRAGMENT))
                        .collect()
                })
                .collect::<Vec<String>>()
                .join(",");
            carrier.set(CORRELATION_CONTEXT_HEADER, header_value);
        }
    }

    /// Extracts a `Context` with correlation context values from a `Carrier`.
    fn extract_with_context(&self, cx: &Context, carrier: &dyn api::Carrier) -> Context {
        if let Some(header_value) = carrier.get(CORRELATION_CONTEXT_HEADER) {
            let correlations = header_value.split(',').flat_map(|context_value| {
                if let Some((name_and_value, props)) = context_value
                    .split(';')
                    .collect::<Vec<&str>>()
                    .split_first()
                {
                    let mut iter = name_and_value.split('=');
                    if let (Some(name), Some(value)) = (iter.next(), iter.next()) {
                        let name = percent_decode_str(name).decode_utf8().map_err(|_| ())?;
                        let value = percent_decode_str(value).decode_utf8().map_err(|_| ())?;

                        // TODO: handle props from https://w3c.github.io/correlation-context/
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
                    // Invalid correlation context value format
                    Err(())
                }
            });
            cx.with_correlations(correlations)
        } else {
            cx.clone()
        }
    }
}

struct Correlations(CorrelationContext);

/// Methods for soring and retrieving correlation data in a context.
pub trait CorrelationContextExt {
    /// Returns a clone of the current context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, CorrelationContextExt, KeyValue, Value};
    ///
    /// let cx = Context::current_with_correlations(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.correlation_context().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn current_with_correlations<T: IntoIterator<Item = KeyValue>>(correlations: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, CorrelationContextExt, KeyValue, Value};
    ///
    /// let some_context = Context::current();
    /// let cx = some_context.with_correlations(vec![KeyValue::new("my-name", "my-value")]);
    ///
    /// assert_eq!(
    ///     cx.correlation_context().get("my-name"),
    ///     Some(&Value::String("my-value".to_string())),
    /// )
    /// ```
    fn with_correlations<T: IntoIterator<Item = KeyValue>>(&self, correlations: T) -> Self;

    /// Returns a clone of the given context with the included name / value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::api::{Context, CorrelationContextExt, KeyValue, Value};
    ///
    /// let cx = Context::current().with_cleared_correlations();
    ///
    /// assert_eq!(cx.correlation_context().len(), 0);
    /// ```
    fn with_cleared_correlations(&self) -> Self;

    /// Returns a reference to this context's correlation context, or the default
    /// empty correlation context if none has been set.
    fn correlation_context(&self) -> &CorrelationContext;
}

impl CorrelationContextExt for Context {
    fn current_with_correlations<T: IntoIterator<Item = KeyValue>>(kvs: T) -> Self {
        Context::current().with_correlations(kvs)
    }

    fn with_correlations<T: IntoIterator<Item = KeyValue>>(&self, kvs: T) -> Self {
        let merged = self
            .correlation_context()
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .chain(kvs.into_iter())
            .collect();

        self.with_value(Correlations(merged))
    }

    fn with_cleared_correlations(&self) -> Self {
        self.with_value(Correlations(CorrelationContext::new()))
    }

    fn correlation_context(&self) -> &CorrelationContext {
        self.get::<Correlations>()
            .map(|correlations| &correlations.0)
            .unwrap_or_else(|| &DEFAULT_CORRELATION_CONTEXT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::HttpTextFormat;
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
	    // "values of non-string types"
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
        ]
    }

    #[test]
    fn extract_correlations() {
        let propagator = CorrelationContextPropagator::new();

        for (header_value, kvs) in valid_extract_data() {
            let mut carrier: HashMap<String, String> = HashMap::new();
            carrier.insert(
                CORRELATION_CONTEXT_HEADER.to_string(),
                header_value.to_string(),
            );
            let context = propagator.extract(&carrier);
            let correlations = context.correlation_context();

            assert_eq!(kvs.len(), correlations.len());
            for (key, value) in correlations {
                assert_eq!(Some(value), kvs.get(key))
            }
        }
    }

    #[test]
    fn inject_correlations() {
        let propagator = CorrelationContextPropagator::new();

        for (kvs, header_parts) in valid_inject_data() {
            let mut carrier = HashMap::new();
            let cx = Context::current_with_correlations(kvs);
            propagator.inject_context(&cx, &mut carrier);
            let header_value = carrier.get(CORRELATION_CONTEXT_HEADER).unwrap();

            assert_eq!(header_parts.join(",").len(), header_value.len(),);
            for header_part in &header_parts {
                assert!(header_value.contains(header_part),)
            }
        }
    }
}
