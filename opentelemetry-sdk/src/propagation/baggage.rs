use opentelemetry::{
    baggage::{Baggage, BaggageExt, BaggageMetadata},
    otel_warn,
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    Context, Key, StringValue,
};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use std::fmt;
use std::sync::OnceLock;
use thiserror::Error;

static BAGGAGE_HEADER: &str = "baggage";

/// Characters that are percent-encoded in values: everything outside
/// `baggage-octet`, plus '%' itself. `CONTROLS` covers 0x00-0x1F and 0x7F, and
/// `utf8_percent_encode` always encodes non-ASCII bytes.
/// See https://www.w3.org/TR/baggage/#value
const VALUE_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b',')
    .add(b';')
    .add(b'\\')
    .add(b'%');

// W3C Baggage specification limits.
// See https://www.w3.org/TR/baggage/#limits
const MAX_BAGGAGE_LENGTH: usize = 8192;
const MAX_BAGGAGE_ITEMS: usize = 64;

// TODO Replace this with LazyLock once it is stable.
static BAGGAGE_FIELDS: OnceLock<[String; 1]> = OnceLock::new();
#[inline]
fn baggage_fields() -> &'static [String; 1] {
    BAGGAGE_FIELDS.get_or_init(|| [BAGGAGE_HEADER.to_owned()])
}

/// Propagates name-value pairs in [W3C Baggage] format.
///
/// Baggage is used to annotate telemetry, adding context and
/// information to metrics, traces and logs. It is an abstract data type
/// represented by a set of name-value pairs describing user defined properties.
/// Each name in a [`Baggage`] is associated with exactly one value.
/// `Baggage`s are serialized according to the [W3C Baggage] specification.
///
/// Values are percent-decoded on extraction and percent-encoded on injection.
/// Names and [`BaggageMetadata`] are propagated verbatim in their wire form,
/// without any decoding or encoding; only optional whitespace surrounding the
/// metadata is trimmed. Malformed list-members are dropped on extraction and
/// entries whose name is not a valid token or whose metadata is not valid W3C
/// `property` syntax are dropped on injection.
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
/// [`BaggageMetadata`]: opentelemetry::baggage::BaggageMetadata
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
        let mut header = String::new();
        let mut invalid = Dropped::default();
        let mut limit_exceeded = false;

        for (name, (value, metadata)) in cx.baggage().iter().take(MAX_BAGGAGE_ITEMS) {
            // Serialize the member straight into the header and roll back to
            // `start` if it cannot be serialized.
            let start = header.len();
            if start > 0 {
                header.push(',');
            }
            match serialize_member(&mut header, name, value, metadata) {
                Ok(()) if header.len() <= MAX_BAGGAGE_LENGTH => continue,
                Ok(()) => limit_exceeded = true,
                Err(err) => invalid.record(err),
            }
            header.truncate(start);
        }

        if invalid.count > 0 || limit_exceeded {
            otel_warn!(
                name: "BaggagePropagator.Inject.DroppedEntries",
                message = "Baggage entries with invalid names or metadata or exceeding W3C limits were not injected",
                invalid_entries = invalid.count as i64,
                first_error = invalid.first_error(),
                limit_exceeded = limit_exceeded,
                limit_bytes = MAX_BAGGAGE_LENGTH as i64,
            );
        }

        if !header.is_empty() {
            injector.set(BAGGAGE_HEADER, header);
        }
    }

    /// Extracts a `Context` with baggage values from a `Extractor`.
    ///
    /// All `baggage` headers are combined into one list. If no valid
    /// list-member is found, the given context is returned unchanged.
    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        let Some(headers) = extractor.get_all(BAGGAGE_HEADER) else {
            return cx.clone();
        };

        let mut baggage = Baggage::new();
        let mut invalid = Dropped::default();
        let mut count_limited = false;

        // Limits apply to the combination of all headers.
        let mut consumed = 0;
        let members = headers
            .iter()
            .filter(|h| !h.is_empty())
            .flat_map(|h| h.split(','))
            .take_while(|raw| {
                consumed += raw.len() + 1;
                consumed <= MAX_BAGGAGE_LENGTH + 1
            });

        for raw in members {
            match parse_member(raw) {
                Ok(member)
                    if baggage.len() == MAX_BAGGAGE_ITEMS && baggage.get(member.name).is_none() =>
                {
                    count_limited = true;
                    break;
                }
                // Last occurrence wins in case of duplicates
                Ok(member) => {
                    // Well-formed escapes that are not valid UTF-8 are
                    // replaced with U+FFFD.
                    let value = percent_decode_str(member.value)
                        .decode_utf8_lossy()
                        .into_owned();
                    baggage.insert_with_metadata(member.name.to_owned(), value, member.metadata);
                }
                Err(err) => invalid.record(err),
            }
        }
        let limit_exceeded = count_limited || consumed > MAX_BAGGAGE_LENGTH + 1;

        if invalid.count > 0 || limit_exceeded {
            otel_warn!(
                name: "BaggagePropagator.Extract.DroppedMembers",
                message = "Dropped baggage list-members that were invalid or exceeded W3C limits",
                invalid_members = invalid.count as i64,
                first_error = invalid.first_error(),
                limit_exceeded = limit_exceeded,
                kept_members = baggage.len() as i64,
                limit_bytes = MAX_BAGGAGE_LENGTH as i64,
                limit_members = MAX_BAGGAGE_ITEMS as i64,
            );
        }

        if baggage.is_empty() {
            cx.clone()
        } else {
            cx.with_baggage(baggage)
        }
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(baggage_fields())
    }
}

/// Reason a baggage entry could not be injected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
enum SerializeError {
    #[error("name is not a valid token")]
    InvalidName,
    #[error("metadata is not valid W3C property syntax")]
    InvalidMetadata,
}

/// Serializes a single `list-member` into `out`. Inverse of [`parse_member`].
fn serialize_member(
    out: &mut String,
    name: &Key,
    value: &StringValue,
    metadata: &BaggageMetadata,
) -> Result<(), SerializeError> {
    // `Baggage` does not restrict names or metadata, so both are validated
    // before anything is written.
    if !validate_token(name.as_str()) {
        return Err(SerializeError::InvalidName);
    }
    let metadata = trim_ows(metadata.as_str());
    if !metadata.is_empty() && !validate_properties(metadata) {
        return Err(SerializeError::InvalidMetadata);
    }

    out.push_str(name.as_str());
    out.push('=');
    out.extend(utf8_percent_encode(value.as_str(), VALUE_ENCODE_SET));
    if !metadata.is_empty() {
        out.push(';');
        out.push_str(metadata);
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct ParsedMember<'a> {
    /// Token taken literally from the wire.
    name: &'a str,
    /// Validated value, still percent encoded.
    value: &'a str,
    /// Properties as received (still percent encoded), surrounding optional
    /// whitespace trimmed.
    metadata: &'a str,
}

/// Reason a list-member was rejected during extraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
enum ParseError {
    #[error("empty list-member")]
    Empty,
    #[error("list-member is missing '='")]
    MissingEquals,
    #[error("name is not a valid token")]
    InvalidName,
    #[error("value is not a valid percent-encoded baggage value")]
    InvalidValue,
    #[error("properties are not valid W3C property syntax")]
    InvalidProperties,
}

/// Parses a single `list-member`.
/// `key OWS "=" OWS value *( OWS ";" OWS property )`
fn parse_member(raw: &str) -> Result<ParsedMember<'_>, ParseError> {
    let raw = trim_ows(raw);
    if raw.is_empty() {
        return Err(ParseError::Empty);
    }

    let (name_value, properties) = match raw.split_once(';') {
        Some((name_value, properties)) => (name_value, Some(properties)),
        None => (raw, None),
    };
    // '=' is not a valid token character but is valid inside values, so only
    // the first '=' separates the name from the value.
    let (name, value) = name_value
        .split_once('=')
        .ok_or(ParseError::MissingEquals)?;

    let name = parse_token(name).ok_or(ParseError::InvalidName)?;

    let value = trim_ows(value);
    if !validate_value(value) {
        return Err(ParseError::InvalidValue);
    }

    let metadata = match properties.map(trim_ows) {
        Some(properties) if !validate_properties(properties) => {
            return Err(ParseError::InvalidProperties)
        }
        Some(properties) => properties,
        None => "",
    };

    Ok(ParsedMember {
        name,
        value,
        metadata,
    })
}

/// Trims optional whitespace and returns `s` if it is a `token`.
fn parse_token(s: &str) -> Option<&str> {
    let s = trim_ows(s);
    validate_token(s).then_some(s)
}

/// Returns `true` if `raw` is a `;` separated list of properties.
/// Optional whitespace is allowed, empty properties are not.
///
/// `property = key OWS "=" OWS value / key OWS`
fn validate_properties(raw: &str) -> bool {
    raw.split(';')
        .map(trim_ows)
        .all(|property| match property.split_once('=') {
            Some((key, value)) => validate_token(trim_ows(key)) && validate_value(trim_ows(value)),
            None => validate_token(property),
        })
}

/// Returns `true` if `s` is a `token` (`1*tchar`).
fn validate_token(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(is_tchar)
}

/// Returns `true` if `s` is `*baggage-octet` and every '%' starts a
/// well-formed percent-encoded octet.
fn validate_value(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if !bytes
                    .get(i + 1..i + 3)
                    .is_some_and(|hex| hex.iter().all(u8::is_ascii_hexdigit))
                {
                    return false;
                }
                i += 3;
            }
            b if is_baggage_octet(b) => i += 1,
            _ => return false,
        }
    }
    true
}

/// Trims optional whitespace (`OWS = *( SP / HTAB )`).
fn trim_ows(s: &str) -> &str {
    s.trim_matches([' ', '\t'])
}

/// `tchar`, see https://datatracker.ietf.org/doc/html/rfc7230#section-3.2.6
const fn is_tchar(b: u8) -> bool {
    matches!(
        b,
        b'0'..=b'9'
            | b'A'..=b'Z'
            | b'^'..=b'z' // ^ _ ` a-z
            | b'!'
            | b'#'..=b'\'' // # $ % & '
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'|'
            | b'~'
    )
}

/// `baggage-octet`: US-ASCII excluding CTLs, whitespace, DQUOTE, comma,
/// semicolon and backslash.
///
/// ```text
/// baggage-octet = %x21 / %x23-2B / %x2D-3A / %x3C-5B / %x5D-7E
/// ```
///
/// See https://www.w3.org/TR/baggage/#value
const fn is_baggage_octet(b: u8) -> bool {
    matches!(b, 0x21 | 0x23..=0x2B | 0x2D..=0x3A | 0x3C..=0x5B | 0x5D..=0x7E)
}

/// Number of invalid members and the first error encountered.
#[derive(Debug)]
struct Dropped<E> {
    count: usize,
    first_error: Option<E>,
}

impl<E> Default for Dropped<E> {
    fn default() -> Self {
        Dropped {
            count: 0,
            first_error: None,
        }
    }
}

impl<E: fmt::Display> Dropped<E> {
    /// Records an invalid member, keeping `error` if it is the first one.
    fn record(&mut self, error: E) {
        self.count += 1;
        self.first_error.get_or_insert(error);
    }

    /// The first error's message or an empty string if there was none.
    fn first_error(&self) -> String {
        self.first_error
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{baggage::KeyValueMetadata, KeyValue, StringValue, Value};
    use std::collections::HashMap;

    /// Extractor that returns several `baggage` headers.
    struct MultiHeaderExtractor(Vec<String>);

    impl Extractor for MultiHeaderExtractor {
        fn get(&self, key: &str) -> Option<&str> {
            self.get_all(key)?.first().copied()
        }

        fn keys(&self) -> Vec<&str> {
            vec![BAGGAGE_HEADER]
        }

        fn get_all(&self, key: &str) -> Option<Vec<&str>> {
            (key == BAGGAGE_HEADER && !self.0.is_empty())
                .then(|| self.0.iter().map(String::as_str).collect())
        }
    }

    fn extract_headers(cx: &Context, headers: &[&str]) -> Context {
        let extractor = MultiHeaderExtractor(headers.iter().map(|h| h.to_string()).collect());
        BaggagePropagator::new().extract_with_context(cx, &extractor)
    }

    fn extract(header: &str) -> Context {
        extract_headers(&Context::new(), &[header])
    }

    fn inject(baggage: Baggage) -> Option<String> {
        let mut injector = HashMap::new();
        BaggagePropagator::new()
            .inject_context(&Context::new().with_baggage(baggage), &mut injector);
        injector.remove(BAGGAGE_HEADER)
    }

    /// Baggage as `name -> (value, metadata)` for easy comparison.
    fn entries(cx: &Context) -> HashMap<String, (String, String)> {
        cx.baggage()
            .iter()
            .map(|(name, (value, metadata))| {
                (
                    name.to_string(),
                    (value.to_string(), metadata.as_str().to_string()),
                )
            })
            .collect()
    }

    fn entries_of(members: &[(&str, &str, &str)]) -> HashMap<String, (String, String)> {
        members
            .iter()
            .map(|(name, value, metadata)| {
                (name.to_string(), (value.to_string(), metadata.to_string()))
            })
            .collect()
    }

    /// Expected `(name, value, metadata)` of an extracted member.
    type Member = (&'static str, &'static str, &'static str);

    #[rustfmt::skip]
    fn extract_cases() -> Vec<(&'static str, Vec<Member>)> {
        vec![
            ("key1=val1,key2=val2", vec![("key1", "val1", ""), ("key2", "val2", "")]),
            ("key1=,key2=val2", vec![("key1", "", ""), ("key2", "val2", "")]),
            ("key1 =   val1,  key2 =val2   ", vec![("key1", "val1", ""), ("key2", "val2", "")]),
            ("key1=val1\t,\tkey2=val2", vec![("key1", "val1", ""), ("key2", "val2", "")]),
            ("key1=val1,key2=val2%2Cval3", vec![("key1", "val1", ""), ("key2", "val2,val3", "")]),
            ("key=%3B%22%5C%25", vec![("key", ";\"\\%", "")]),
            ("key=a+b", vec![("key", "a+b", "")]),
            ("key1=val1,key2=val2;prop=1", vec![("key1", "val1", ""), ("key2", "val2", "prop=1")]),
            ("key1=val1,key2=val2;prop1", vec![("key1", "val1", ""), ("key2", "val2", "prop1")]),
            (
                "key1=value1;property1;property2, key2 = value2, key3=value3; propertyKey=propertyValue",
                vec![
                    ("key1", "value1", "property1;property2"),
                    ("key2", "value2", ""),
                    ("key3", "value3", "propertyKey=propertyValue"),
                ],
            ),
            ("key=1 ; p = x%2Cy ; flag ", vec![("key", "1", "p = x%2Cy ; flag")]),
            ("key=1;p=", vec![("key", "1", "p=")]),
            ("key=1;p=a=b", vec![("key", "1", "p=a=b")]),
            ("key1=val1,key2=val2,a,val3", vec![("key1", "val1", ""), ("key2", "val2", "")]),
            ("a=1,,b=2,", vec![("a", "1", ""), ("b", "2", "")]),
            ("a=1,=2", vec![("a", "1", "")]),
            ("a=1,b c=2", vec![("a", "1", "")]),
            ("a=1,(b)=2", vec![("a", "1", "")]),
            ("a=1,ké=2", vec![("a", "1", "")]),
            ("a=1,b=x y", vec![("a", "1", "")]),
            ("a=1,b=\"x\"", vec![("a", "1", "")]),
            ("a=1,b=x\\y", vec![("a", "1", "")]),
            ("a=1,b=é", vec![("a", "1", "")]),
            ("a=1,b=%ZZ", vec![("a", "1", "")]),
            ("a=1,b=%4", vec![("a", "1", "")]),
            ("a=1,b=%", vec![("a", "1", "")]),
            ("a=1,b=2;p=%ZZ", vec![("a", "1", "")]),
            ("a=1,b=2;p q", vec![("a", "1", "")]),
            ("a=1,b=2;(p)", vec![("a", "1", "")]),
            ("a=1,b=2;p=x y", vec![("a", "1", "")]),
            ("a=1,b=2;=x", vec![("a", "1", "")]),
            ("a=1,b=2;", vec![("a", "1", "")]),
            ("a=1,b=2; ", vec![("a", "1", "")]),
            ("a=1,b=2;;p", vec![("a", "1", "")]),
            ("a=1,b=2;p;", vec![("a", "1", "")]),
            // duplicate names: last wins
            ("a=1,a=2", vec![("a", "2", "")]),
        ]
    }

    #[test]
    fn extract_follows_grammar() {
        for (header, members) in extract_cases() {
            assert_eq!(
                entries(&extract(header)),
                entries_of(&members),
                "{header:?}"
            );
        }
    }

    #[test]
    fn validate_token_accepts_tokens() {
        for valid in ["a", "k%41", "key-1.x_y~"] {
            assert!(validate_token(valid), "{valid:?}");
        }
        for invalid in ["", " a", "a ", "a b", "ké", "(a)", "a=b"] {
            assert!(!validate_token(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn validate_value_accepts_baggage_octets() {
        for valid in ["", "abc", "%2C", "a=b"] {
            assert!(validate_value(valid), "{valid:?}");
        }
        for invalid in [" a", "a b", "%", "%4", "%ZZ", "é", "\"x\"", "a,b", "a;b"] {
            assert!(!validate_value(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn validate_properties_follows_grammar() {
        for valid in ["p", "p=", "p=a=b", "p = x%2C ; f", "a;b=1;c"] {
            assert!(validate_properties(valid), "{valid:?}");
        }
        for invalid in [
            "", " ", ";", "p;", ";p", "p;;q", "p=x y", "p=%ZZ", "=x", "p q",
        ] {
            assert!(!validate_properties(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn is_tchar_matches_rfc7230() {
        for b in 0..=255u8 {
            let expected = b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b);
            assert_eq!(is_tchar(b), expected, "byte {b:#04x}");
        }
    }

    #[test]
    fn value_encode_set_matches_baggage_octet() {
        // Injection must encode exactly the bytes outside `baggage-octet`,
        // plus '%' itself.
        for b in 0..0x80u8 {
            let s = (b as char).to_string();
            let encoded = utf8_percent_encode(&s, VALUE_ENCODE_SET).to_string() != s;
            assert_eq!(encoded, !is_baggage_octet(b) || b == b'%', "byte {b:#04x}");
        }
    }

    #[test]
    fn parse_member_reports_error_kind() {
        let cases = [
            ("", ParseError::Empty),
            (" \t ", ParseError::Empty),
            ("a", ParseError::MissingEquals),
            ("a;p=1", ParseError::MissingEquals),
            ("=1", ParseError::InvalidName),
            ("a b=1", ParseError::InvalidName),
            ("a=x y", ParseError::InvalidValue),
            ("a=%G0", ParseError::InvalidValue),
            ("a=1;p q", ParseError::InvalidProperties),
            ("a=1;p=%ZZ", ParseError::InvalidProperties),
            ("a=1;", ParseError::InvalidProperties),
            ("a=1;p;;q", ParseError::InvalidProperties),
        ];
        for (raw, err) in cases {
            assert_eq!(parse_member(raw), Err(err), "{raw:?}");
        }
    }

    #[rustfmt::skip]
    fn inject_cases() -> Vec<(Vec<KeyValueMetadata>, Vec<&'static str>)> {
        vec![
            (vec![KeyValue::new("key1", "val1").into(), KeyValue::new("key2", "val2").into()], vec!["key1=val1", "key2=val2"]),
            (vec![KeyValue::new("key1", "val1,val2").into(), KeyValue::new("key2", "val3=4").into()], vec!["key1=val1%2Cval2", "key2=val3=4"]),
            (
                vec![
                    KeyValue::new("key1", true).into(),
                    KeyValue::new("key2", Value::I64(123)).into(),
                    KeyValue::new("key3", Value::F64(123.567)).into(),
                ],
                vec!["key1=true", "key2=123", "key3=123.567"],
            ),
            (
                vec![
                    KeyValue::new("key1", Value::Array(vec![true, false].into())).into(),
                    KeyValue::new("key2", Value::Array(vec![123, 456].into())).into(),
                    KeyValue::new("key3", Value::Array(vec![StringValue::from("val1"), StringValue::from("val2")].into())).into(),
                ],
                vec!["key1=[true%2Cfalse]", "key2=[123%2C456]", "key3=[%22val1%22%2C%22val2%22]"],
            ),
            (
                vec![
                    KeyValueMetadata::new("key1", "val1", "prop1"),
                    KeyValue::new("key2", "val2").into(),
                    KeyValueMetadata::new("key3", "val3", "anykey=anyvalue"),
                    KeyValueMetadata::new("key4", "val4", " p = 1 ; f "),
                    KeyValueMetadata::new("key5", "val5", " \t "),
                ],
                vec!["key1=val1;prop1", "key2=val2", "key3=val3;anykey=anyvalue", "key4=val4;p = 1 ; f", "key5=val5"],
            ),
            (
                vec![
                    KeyValueMetadata::new("key1", "val1", "p=x,admin=true"),
                    KeyValueMetadata::new("key2", "val2", "p=a b"),
                    KeyValueMetadata::new("key3", "val3", "p=é"),
                    KeyValueMetadata::new("key4", "val4", "p=%"),
                    KeyValueMetadata::new("key5", "val5", "ok"),
                    KeyValueMetadata::new("key6", "val6", " p = 1 ;; f "),
                    KeyValueMetadata::new("key7", "val7", " ; "),
                    KeyValueMetadata::new("key8", "val8", "p;"),
                ],
                vec!["key5=val5;ok"],
            ),
        ]
    }

    #[test]
    fn inject_produces_expected_members() {
        for (members, header_parts) in inject_cases() {
            let header = inject(members.into()).unwrap();
            let mut actual: Vec<&str> = header.split(',').collect();
            actual.sort_unstable();
            let mut expected = header_parts.clone();
            expected.sort_unstable();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn inject_nothing_for_empty_baggage() {
        assert_eq!(inject(Baggage::new()), None);
        // only entries that cannot be propagated
        assert_eq!(
            inject([KeyValueMetadata::new("k", "v", "a,b")].into()),
            None
        );
    }

    #[test]
    fn inject_skips_names_that_are_not_tokens() {
        let baggage = [
            KeyValue::new("Grüße", "1"),
            KeyValue::new("(a)", "2"),
            KeyValue::new("a b", "3"),
            KeyValue::new("ok", "4"),
        ]
        .into_iter()
        .collect::<Baggage>();
        assert_eq!(baggage.len(), 4);
        assert_eq!(inject(baggage).as_deref(), Some("ok=4"));

        assert_eq!(inject([KeyValue::new("a,b", "1")].into()), None);
    }

    #[test]
    fn properties_keep_encoding_through_extract_and_inject() {
        let cx = extract("a=1;p=x%2Cadmin%3Dtrue");
        assert_eq!(
            entries(&cx),
            entries_of(&[("a", "1", "p=x%2Cadmin%3Dtrue")])
        );

        let mut injector = HashMap::new();
        BaggagePropagator::new().inject_context(&cx, &mut injector);
        assert_eq!(
            injector.get(BAGGAGE_HEADER).map(String::as_str),
            Some("a=1;p=x%2Cadmin%3Dtrue")
        );
    }

    #[test]
    fn equals_signs_in_values_are_preserved() {
        assert_eq!(entries(&extract("a=b=c")), entries_of(&[("a", "b=c", "")]));
        assert_eq!(
            entries(&extract("token=abc==")),
            entries_of(&[("token", "abc==", "")])
        );
    }

    #[test]
    fn inject_encodes_percent_and_backslash() {
        let header = inject([KeyValue::new("a", "100% \\ok")].into()).unwrap();
        assert_eq!(header, "a=100%25%20%5Cok");
        // a literal '%' that looks like an escape must not be decoded later
        let header = inject([KeyValue::new("a", "%41")].into()).unwrap();
        assert_eq!(header, "a=%2541");
        assert_eq!(entries(&extract(&header)), entries_of(&[("a", "%41", "")]));
    }

    #[test]
    fn encoded_whitespace_is_preserved() {
        assert_eq!(
            entries(&extract("a=%20x%20")),
            entries_of(&[("a", " x ", "")])
        );
        assert_eq!(
            entries(&extract("a = %09x ")),
            entries_of(&[("a", "\tx", "")])
        );
        assert_eq!(
            inject([KeyValue::new("a", " x ")].into()).as_deref(),
            Some("a=%20x%20")
        );
    }

    #[test]
    fn names_are_not_percent_decoded() {
        // '%' is a literal token character, so these are two distinct names
        assert_eq!(
            entries(&extract("k%41=1,kA=2")),
            entries_of(&[("k%41", "1", ""), ("kA", "2", "")])
        );
        // names that are only valid after decoding are rejected
        assert!(extract("k%20x=1").baggage().get("k x").is_none());
        assert_eq!(
            entries(&extract("k%20x=1")),
            entries_of(&[("k%20x", "1", "")])
        );
    }

    #[test]
    fn repeated_headers_are_combined() {
        let cx = extract_headers(&Context::new(), &["a=1", "b=2,c=3", "", "a=4"]);
        assert_eq!(
            entries(&cx),
            entries_of(&[("a", "4", ""), ("b", "2", ""), ("c", "3", "")])
        );
    }

    #[test]
    fn invalid_utf8_is_replaced() {
        assert_eq!(
            entries(&extract("a=%FF,b=x%C3,c=%E2%82%AC")),
            entries_of(&[
                ("a", "\u{FFFD}", ""),
                ("b", "x\u{FFFD}", ""),
                ("c", "€", "")
            ])
        );
    }

    #[test]
    fn existing_baggage_is_retained() {
        let cx = Context::new().with_baggage([KeyValue::new("existing", "1")]);
        let existing = entries_of(&[("existing", "1", "")]);

        assert_eq!(entries(&extract_headers(&cx, &[])), existing);
        assert_eq!(entries(&extract_headers(&cx, &[""])), existing);
        assert_eq!(entries(&extract_headers(&cx, &["bad, =1,a b=2"])), existing);

        // valid baggage replaces the existing baggage
        assert_eq!(
            entries(&extract_headers(&cx, &["new=2"])),
            entries_of(&[("new", "2", "")])
        );
    }

    #[test]
    fn inject_limit_uses_encoded_size() {
        // "k=" + 8190 bytes is exactly the limit
        let fits = "x".repeat(MAX_BAGGAGE_LENGTH - 2);
        let header = inject([KeyValue::new("k", fits)].into()).unwrap();
        assert_eq!(header.len(), MAX_BAGGAGE_LENGTH);

        let too_long = "x".repeat(MAX_BAGGAGE_LENGTH - 1);
        assert_eq!(inject([KeyValue::new("k", too_long)].into()), None);

        // 2731 spaces are below the limit when stored, but encode to 8193 bytes
        let spaces = " ".repeat(2731);
        let header =
            inject([KeyValue::new("big", spaces), KeyValue::new("small", "1")].into()).unwrap();
        assert_eq!(header, "small=1");
    }

    #[test]
    fn values_and_metadata_round_trip() {
        let values = [
            "",
            "plain",
            ", ; = % \\ \" space\ttab",
            "é 😊",
            "  leading and trailing  ",
            "%41%",
            "a+b",
            "\u{FFFD}",
        ];
        let metadata = ["", "flag", "p=x%2Cy;flag;q=%25", "p=a=b"];

        let members: Vec<KeyValueMetadata> = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                KeyValueMetadata::new(format!("key{i}"), *value, metadata[i % metadata.len()])
            })
            .collect();
        let original = entries(&Context::new().with_baggage(members.clone()));
        assert_eq!(original.len(), values.len());

        let extracted = extract(&inject(members.into()).unwrap());
        assert_eq!(entries(&extracted), original);

        // and once more from the extracted context
        let mut injector = HashMap::new();
        BaggagePropagator::new().inject_context(&extracted, &mut injector);
        assert_eq!(entries(&extract(&injector[BAGGAGE_HEADER])), original);
    }

    #[test]
    fn extract_keeps_members_within_max_length() {
        // Build a syntactically valid header longer than the W3C 8192-byte
        // limit. Members that fit entirely within the limit are kept.
        let mut header = String::with_capacity(MAX_BAGGAGE_LENGTH + 1024);
        let mut kept = 0;
        let mut i = 0u32;
        while header.len() < MAX_BAGGAGE_LENGTH + 256 {
            if !header.is_empty() {
                header.push(',');
            }
            header.push_str(&format!("k{i}={}", "v".repeat(200)));
            if header.len() <= MAX_BAGGAGE_LENGTH {
                kept += 1;
            }
            i += 1;
        }
        assert!(header.len() > MAX_BAGGAGE_LENGTH);

        let cx = extract(&header);
        assert_eq!(cx.baggage().len(), kept);
        assert!(cx.baggage().get("k0").is_some());
        assert!(cx.baggage().get(format!("k{kept}")).is_none());
    }

    #[test]
    fn extract_length_limit_applies_across_headers() {
        let half = format!("a={}", "x".repeat(MAX_BAGGAGE_LENGTH / 2));
        let other = format!("b={}", "x".repeat(MAX_BAGGAGE_LENGTH / 2));
        let cx = extract_headers(&Context::new(), &[&half, &other]);
        assert_eq!(cx.baggage().len(), 1);
        assert!(cx.baggage().get("a").is_some());
    }

    #[test]
    fn extract_caps_entry_count_at_max_baggage_items() {
        // Build a header whose total size stays under MAX_BAGGAGE_LENGTH but
        // contains more than MAX_BAGGAGE_ITEMS entries.
        let entries = MAX_BAGGAGE_ITEMS + 32;
        let header = (0..entries)
            .map(|i| format!("k{i:03}=v"))
            .collect::<Vec<_>>()
            .join(",");
        assert!(header.len() <= MAX_BAGGAGE_LENGTH);

        let context = extract(&header);
        assert_eq!(context.baggage().len(), MAX_BAGGAGE_ITEMS);
    }

    #[test]
    fn extract_entry_count_applies_across_headers() {
        let first = (0..40)
            .map(|i| format!("a{i}=v"))
            .collect::<Vec<_>>()
            .join(",");
        let second = (0..40)
            .map(|i| format!("b{i}=v"))
            .collect::<Vec<_>>()
            .join(",");
        let cx = extract_headers(&Context::new(), &[&first, &second]);
        assert_eq!(cx.baggage().len(), MAX_BAGGAGE_ITEMS);
        assert!(cx.baggage().get("b23").is_some());
        assert!(cx.baggage().get("b24").is_none());
    }

    #[test]
    fn extract_updates_existing_name_at_entry_limit() {
        let header = (0..MAX_BAGGAGE_ITEMS)
            .map(|i| format!("k{i}=v"))
            .chain(["k0=updated".to_string()])
            .collect::<Vec<_>>()
            .join(",");
        let cx = extract(&header);
        assert_eq!(cx.baggage().len(), MAX_BAGGAGE_ITEMS);
        assert_eq!(cx.baggage().get("k0"), Some(&StringValue::from("updated")));
    }
}
