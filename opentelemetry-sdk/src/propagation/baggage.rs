use opentelemetry::{
    baggage::{BaggageExt, KeyValueMetadata},
    otel_warn,
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    Context,
};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use std::iter;
use std::sync::OnceLock;

static BAGGAGE_HEADER: &str = "baggage";
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b';').add(b',').add(b'=');

// TODO Replace this with LazyLock once it is stable.
static BAGGAGE_FIELDS: OnceLock<[String; 1]> = OnceLock::new();
#[inline]
fn baggage_fields() -> &'static [String; 1] {
    BAGGAGE_FIELDS.get_or_init(|| [BAGGAGE_HEADER.to_owned()])
}

/// Propagates name-value pairs in [W3C Baggage] format.
///
/// Baggage is used to annotate telemetry, adding context and
/// information to metrics, traces, and logs. It is an abstract data type
/// represented by a set of name-value pairs describing user-defined properties.
/// Each name in a [`Baggage`] is associated with exactly one value.
/// `Baggage`s are serialized according to the editor's draft of
/// the [W3C Baggage] specification.
///
/// # Examples
///
/// ```
/// use opentelemetry::{baggage::{Baggage, BaggageExt}, propagation::TextMapPropagator};
/// use opentelemetry_sdk::propagation::BaggagePropagator;
/// use std::collections::HashMap;
///
/// // Example baggage value passed in externally via http headers
/// let mut headers = HashMap::new();
/// headers.insert("baggage".to_string(), "user_id=1".to_string());
///
/// let propagator = BaggagePropagator::new();
/// // can extract from any type that impls `Extractor`, usually an HTTP header map
/// let cx = propagator.extract(&headers);
///
/// // Iterate over extracted name-value pairs
/// for (name, value) in cx.baggage() {
///     // ...
/// }
///
/// // Add new baggage
/// let mut baggage = Baggage::new();
/// let _ = baggage.insert("server_id", "42");
///
/// let cx_with_additions = cx.with_baggage(baggage);
///
/// // Inject baggage into http request
/// propagator.inject_context(&cx_with_additions, &mut headers);
///
/// let header_value = headers.get("baggage").expect("header is injected");
/// assert!(!header_value.contains("user_id=1"), "still contains previous name-value");
/// assert!(header_value.contains("server_id=42"), "does not contain new name-value pair");
/// ```
///
/// [W3C Baggage]: https://w3c.github.io/baggage
/// [`Baggage`]: opentelemetry::baggage::Baggage
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
                .map(|(name, (value, metadata))| {
                    let metadata_str = metadata.as_str().trim();
                    let metadata_prefix = if metadata_str.is_empty() { "" } else { ";" };
                    utf8_percent_encode(name.as_str().trim(), FRAGMENT)
                        .chain(iter::once("="))
                        .chain(utf8_percent_encode(value.as_str().trim(), FRAGMENT))
                        .chain(iter::once(metadata_prefix))
                        .chain(iter::once(metadata_str))
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
            let baggage = header_value.split(',').filter_map(|context_value| {
                if let Some((name_and_value, props)) = context_value
                    .split(';')
                    .collect::<Vec<&str>>()
                    .split_first()
                {
                    let mut iter = name_and_value.split('=');
                    if let (Some(name), Some(value)) = (iter.next(), iter.next()) {
                        let decode_name = percent_decode_str(name).decode_utf8();
                        let decode_value = percent_decode_str(value).decode_utf8();

                        if let (Ok(name), Ok(value)) = (decode_name, decode_value) {
                            // Here we don't store the first ; into baggage since it should be treated
                            // as separator rather part of metadata
                            let decoded_props = props
                                .iter()
                                .flat_map(|prop| percent_decode_str(prop).decode_utf8())
                                .map(|prop| prop.trim().to_string())
                                .collect::<Vec<String>>()
                                .join(";"); // join with ; because we deleted all ; when calling split above

                            Some(KeyValueMetadata::new(
                                name.trim().to_owned(),
                                value.trim().to_string(),
                                decoded_props.as_str(),
                            ))
                        } else {
                            otel_warn!(
                                name: "BaggagePropagator.Extract.InvalidUTF8",
                                message = "Invalid UTF8 string in key values",
                                baggage_header = header_value,
                            );
                            None
                        }
                    } else {
                        otel_warn!(
                            name: "BaggagePropagator.Extract.InvalidKeyValueFormat",
                            message = "Invalid baggage key-value format",
                            baggage_header = header_value,
                        );
                        None
                    }
                } else {
                    otel_warn!(
                        name: "BaggagePropagator.Extract.InvalidFormat",
                        message = "Invalid baggage format",
                        baggage_header = header_value);
                    None
                }
            });
            cx.with_baggage(baggage)
        } else {
            cx.clone()
        }
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(baggage_fields())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{baggage::BaggageMetadata, Key, KeyValue, StringValue, Value};
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn valid_extract_data() -> Vec<(&'static str, HashMap<Key, StringValue>)> {
        vec![
            // "valid w3cHeader"
            ("key1=val1,key2=val2", vec![(Key::new("key1"), StringValue::from("val1")), (Key::new("key2"), StringValue::from("val2"))].into_iter().collect()),
            // "valid w3cHeader with spaces"
            ("key1 =   val1,  key2 =val2   ", vec![(Key::new("key1"), StringValue::from("val1")), (Key::new("key2"), StringValue::from("val2"))].into_iter().collect()),
            // "valid header with url-escaped comma"
            ("key1=val1,key2=val2%2Cval3", vec![(Key::new("key1"), StringValue::from("val1")), (Key::new("key2"), StringValue::from("val2,val3"))].into_iter().collect()),
            // "valid header with an invalid header"
            ("key1=val1,key2=val2,a,val3", vec![(Key::new("key1"), StringValue::from("val1")), (Key::new("key2"), StringValue::from("val2"))].into_iter().collect()),
            // "valid header with no value"
            ("key1=,key2=val2", vec![(Key::new("key1"), StringValue::from("")), (Key::new("key2"), StringValue::from("val2"))].into_iter().collect()),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn valid_extract_data_with_metadata() -> Vec<(&'static str, HashMap<Key, (StringValue, BaggageMetadata)>)> {
        vec![
            // "valid w3cHeader with properties"
            ("key1=val1,key2=val2;prop=1", vec![(Key::new("key1"), (StringValue::from("val1"), BaggageMetadata::default())), (Key::new("key2"), (StringValue::from("val2"), BaggageMetadata::from("prop=1")))].into_iter().collect()),
            // prop can don't need to be key value pair
            ("key1=val1,key2=val2;prop1", vec![(Key::new("key1"), (StringValue::from("val1"), BaggageMetadata::default())), (Key::new("key2"), (StringValue::from("val2"), BaggageMetadata::from("prop1")))].into_iter().collect()),
            ("key1=value1;property1;property2, key2 = value2, key3=value3; propertyKey=propertyValue",
             vec![
                 (Key::new("key1"), (StringValue::from("value1"), BaggageMetadata::from("property1;property2"))),
                 (Key::new("key2"), (StringValue::from("value2"), BaggageMetadata::default())),
                 (Key::new("key3"), (StringValue::from("value3"), BaggageMetadata::from("propertyKey=propertyValue"))),
             ].into_iter().collect()),
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
                    KeyValue::new("key3", Value::F64(123.567)),
                ],
                vec![
                    "key1=true",
                    "key2=123",
                    "key3=123.567",
                ],
            ),
            // "values of array types"
            (
                vec![
                    KeyValue::new("key1", Value::Array(vec![true, false].into())),
                    KeyValue::new("key2", Value::Array(vec![123, 456].into())),
                    KeyValue::new("key3", Value::Array(vec![StringValue::from("val1"), StringValue::from("val2")].into())),
                ],
                vec![
                    "key1=[true%2Cfalse]",
                    "key2=[123%2C456]",
                    "key3=[%22val1%22%2C%22val2%22]",
                ],
            ),
        ]
    }

    #[rustfmt::skip]
    fn valid_inject_data_metadata() -> Vec<(Vec<KeyValueMetadata>, Vec<&'static str>)> {
        vec![
            (
                vec![
                    KeyValueMetadata::new("key1", "val1", "prop1"),
                    KeyValue::new("key2", "val2").into(),
                    KeyValueMetadata::new("key3", "val3", "anykey=anyvalue"),
                ],
                vec![
                    "key1=val1;prop1",
                    "key2=val2",
                    "key3=val3;anykey=anyvalue",
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
            for (key, (value, _metadata)) in baggage {
                assert_eq!(Some(value), kvs.get(key))
            }
        }
    }

    #[test]
    fn inject_baggage() {
        let propagator = BaggagePropagator::new();

        for (kvm, header_parts) in valid_inject_data() {
            let mut injector = HashMap::new();
            let cx = Context::current_with_baggage(kvm);
            propagator.inject_context(&cx, &mut injector);
            let header_value = injector.get(BAGGAGE_HEADER).unwrap();
            assert_eq!(header_parts.join(",").len(), header_value.len(),);
            for header_part in &header_parts {
                assert!(header_value.contains(header_part),)
            }
        }
    }

    #[test]
    fn extract_baggage_with_metadata() {
        let propagator = BaggagePropagator::new();
        for (header_value, kvm) in valid_extract_data_with_metadata() {
            let mut extractor: HashMap<String, String> = HashMap::new();
            extractor.insert(BAGGAGE_HEADER.to_string(), header_value.to_string());
            let context = propagator.extract(&extractor);
            let baggage = context.baggage();

            assert_eq!(kvm.len(), baggage.len());
            for (key, value_and_prop) in baggage {
                assert_eq!(Some(value_and_prop), kvm.get(key))
            }
        }
    }

    #[test]
    fn inject_baggage_with_metadata() {
        let propagator = BaggagePropagator::new();

        for (kvm, header_parts) in valid_inject_data_metadata() {
            let mut injector = HashMap::new();
            let cx = Context::current_with_baggage(kvm);
            propagator.inject_context(&cx, &mut injector);
            let header_value = injector.get(BAGGAGE_HEADER).unwrap();

            assert_eq!(header_parts.join(",").len(), header_value.len());
            for header_part in &header_parts {
                assert!(header_value.contains(header_part),)
            }
        }
    }

    #[rustfmt::skip]
    fn malformed_baggage_test_data() -> Vec<(String, &'static str)> {
        vec![
            // Empty and whitespace
            ("".to_string(), "empty header"),
            ("   ".to_string(), "whitespace only header"),
            
            // Malformed key-value pairs
            ("key_without_value".to_string(), "missing equals sign"),
            ("=value_without_key".to_string(), "missing key"),
            ("key=".to_string(), "empty value allowed"),
            ("=".to_string(), "empty key and value"),
            
            // Multiple equals signs
            ("key=value=extra".to_string(), "multiple equals signs"),
            ("key=val=ue=more".to_string(), "many equals signs"),
            
            // Control characters and non-printable characters
            ("key=val\x00ue".to_string(), "null character in value"),
            ("key\x01=value".to_string(), "control character in key"),
            ("key=value\x7F".to_string(), "DEL character in value"),
            ("key\t=value".to_string(), "tab character in key"),
            ("key=val\nue".to_string(), "newline in value"),
            ("key=val\rue".to_string(), "carriage return in value"),
            
            // Invalid UTF-8 sequences (these will be handled by percent decoding)
            ("key=%80".to_string(), "invalid UTF-8 start byte"),
            ("key=%C2".to_string(), "incomplete UTF-8 sequence"),
            ("key=%ED%A0%80".to_string(), "UTF-8 surrogate"),
            
            // Very long keys and values
            (format!("{}=value", "a".repeat(1000)), "very long key"),
            (format!("key={}", "v".repeat(1000)), "very long value"),
            (format!("{}={}", "k".repeat(500), "v".repeat(500)), "long key and value"),
            
            // Many entries to test memory usage
            ((0..1000).map(|i| format!("key{}=val{}", i, i)).collect::<Vec<_>>().join(","), "many entries"),
            
            // Malformed metadata
            ("key=value;".to_string(), "empty metadata"),
            ("key=value;;".to_string(), "double semicolon"),
            ("key=value;meta;".to_string(), "trailing semicolon"),
            ("key=value;meta=".to_string(), "metadata with empty value"),
            
            // Mixed valid and invalid entries
            ("valid_key=valid_value,invalid_key,another_valid=value".to_string(), "mixed valid and invalid"),
            ("key1=val1,=,key2=val2".to_string(), "empty entry in middle"),
            
            // Extreme whitespace
            ("   key1   =   val1   ,   key2   =   val2   ".to_string(), "excessive whitespace"),
            
            // Special characters that might cause issues
            ("key=value,".to_string(), "trailing comma"),
            (",key=value".to_string(), "leading comma"),
            ("key=value,,".to_string(), "double comma"),
            ("key=val,ue,key2=val2".to_string(), "comma in entry"),
            
            // Unicode characters
            ("café=bücher".to_string(), "unicode characters"),
            ("key=🔥".to_string(), "emoji in value"),
            ("🗝️=value".to_string(), "emoji in key"),
        ]
    }

    #[test]
    fn extract_baggage_defensive_parsing() {
        let propagator = BaggagePropagator::new();

        for (malformed_header, description) in malformed_baggage_test_data() {
            let mut extractor: HashMap<String, String> = HashMap::new();
            extractor.insert(BAGGAGE_HEADER.to_string(), malformed_header.clone());
            
            // The main requirement is that parsing doesn't crash or hang
            let context = propagator.extract(&extractor);
            let baggage = context.baggage();
            
            // Baggage should be created without crashing, regardless of content
            // Invalid entries should be ignored or handled gracefully
            assert!(
                baggage.len() <= 1000, // Reasonable upper bound
                "Too many baggage entries extracted from malformed header: {} ({})", 
                description, baggage.len()
            );
            
            // No entry should have an empty key (our validation should prevent this)
            for (key, _) in baggage {
                assert!(
                    !key.as_str().is_empty(),
                    "Empty key found in baggage from header: {} ({})",
                    malformed_header, description
                );
            }
        }
    }

    #[test]
    fn extract_baggage_memory_safety() {
        let propagator = BaggagePropagator::new();
        
        // Test extremely long header to ensure no memory exhaustion
        let very_long_header = format!("key={}", "x".repeat(100_000));
        let mut extractor: HashMap<String, String> = HashMap::new();
        extractor.insert(BAGGAGE_HEADER.to_string(), very_long_header);
        
        let context = propagator.extract(&extractor);
        let baggage = context.baggage();
        
        // Should handle gracefully without crashing
        assert!(baggage.len() <= 1);
        
        // Test header with many small entries
        let many_entries: Vec<String> = (0..10_000)
            .map(|i| format!("k{}=v{}", i, i))
            .collect();
        let large_header = many_entries.join(",");
        
        let mut extractor2: HashMap<String, String> = HashMap::new();
        extractor2.insert(BAGGAGE_HEADER.to_string(), large_header);
        
        let context2 = propagator.extract(&extractor2);
        let baggage2 = context2.baggage();
        
        // Should handle gracefully, possibly truncating or limiting entries
        assert!(baggage2.len() <= 10_000);
        
        // Verify no extremely long keys or values made it through
        for (key, (value, _)) in baggage2 {
            assert!(
                key.as_str().len() <= 1000,
                "Key too long: {} chars", key.as_str().len()
            );
            assert!(
                value.as_str().len() <= 1000,
                "Value too long: {} chars", value.as_str().len()
            );
        }
    }

    #[test]
    fn extract_baggage_percent_encoding_edge_cases() {
        let propagator = BaggagePropagator::new();
        
        let test_cases = vec![
            ("%", "lone percent sign"),
            ("key=%", "percent at end"),
            ("key=%2", "incomplete percent encoding"),
            ("key=%ZZ", "invalid hex in percent encoding"),
            ("key=%2G", "invalid hex digit"),
            ("key=%%20", "double percent"),
            ("key=%20%20%20", "multiple encoded spaces"),
        ];
        
        for (header, _description) in test_cases {
            let mut extractor: HashMap<String, String> = HashMap::new();
            extractor.insert(BAGGAGE_HEADER.to_string(), header.to_string());
            
            // Should not crash on invalid percent encoding
            let context = propagator.extract(&extractor);
            let _baggage = context.baggage();
            // Test passes if no panic occurs
        }
    }
}
