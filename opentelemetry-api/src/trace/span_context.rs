use crate::trace::{TraceError, TraceResult};
use std::collections::VecDeque;
use std::fmt;
use std::hash::Hash;
use std::num::ParseIntError;
use std::ops::{BitAnd, BitOr, Not};
use std::str::FromStr;
use thiserror::Error;

/// Flags that can be set on a [`SpanContext`].
///
/// The current version of the specification only supports a single flag
/// [`TraceFlags::SAMPLED`].
///
/// See the W3C TraceContext specification's [trace-flags] section for more
/// details.
///
/// [trace-flags]: https://www.w3.org/TR/trace-context/#trace-flags
#[derive(Clone, Debug, Default, PartialEq, Eq, Copy, Hash)]
pub struct TraceFlags(u8);

impl TraceFlags {
    /// Trace flags with the `sampled` flag set to `1`.
    ///
    /// Spans that are not sampled will be ignored by most tracing tools.
    /// See the `sampled` section of the [W3C TraceContext specification] for details.
    ///
    /// [W3C TraceContext specification]: https://www.w3.org/TR/trace-context/#sampled-flag
    pub const SAMPLED: TraceFlags = TraceFlags(0x01);

    /// Construct new trace flags
    pub const fn new(flags: u8) -> Self {
        TraceFlags(flags)
    }

    /// Returns `true` if the `sampled` flag is set
    pub fn is_sampled(&self) -> bool {
        (*self & TraceFlags::SAMPLED) == TraceFlags::SAMPLED
    }

    /// Returns copy of the current flags with the `sampled` flag set.
    pub fn with_sampled(&self, sampled: bool) -> Self {
        if sampled {
            *self | TraceFlags::SAMPLED
        } else {
            *self & !TraceFlags::SAMPLED
        }
    }

    /// Returns the flags as a `u8`
    pub fn to_u8(self) -> u8 {
        self.0
    }
}

impl BitAnd for TraceFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for TraceFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Not for TraceFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl fmt::LowerHex for TraceFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// A 16-byte value which identifies a given trace.
///
/// The id is valid if it contains at least one non-zero byte.
#[derive(Clone, PartialEq, Eq, Copy, Hash)]
pub struct TraceId(pub(crate) u128);

impl TraceId {
    /// Invalid trace id
    pub const INVALID: TraceId = TraceId(0);

    /// Create a trace id from its representation as a byte array.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        TraceId(u128::from_be_bytes(bytes))
    }

    /// Return the representation of this trace id as a byte array.
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Converts a string in base 16 to a trace id.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::trace::TraceId;
    ///
    /// assert!(TraceId::from_hex("42").is_ok());
    /// assert!(TraceId::from_hex("58406520a006649127e371903a2de979").is_ok());
    ///
    /// assert!(TraceId::from_hex("not_hex").is_err());
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, ParseIntError> {
        u128::from_str_radix(hex, 16).map(TraceId)
    }
}

impl From<[u8; 16]> for TraceId {
    fn from(bytes: [u8; 16]) -> Self {
        TraceId::from_bytes(bytes)
    }
}

impl fmt::Debug for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl fmt::LowerHex for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// An 8-byte value which identifies a given span.
///
/// The id is valid if it contains at least one non-zero byte.
#[derive(Clone, PartialEq, Eq, Copy, Hash)]
pub struct SpanId(pub(crate) u64);

impl SpanId {
    /// Invalid span id
    pub const INVALID: SpanId = SpanId(0);

    /// Create a span id from its representation as a byte array.
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        SpanId(u64::from_be_bytes(bytes))
    }

    /// Return the representation of this span id as a byte array.
    pub const fn to_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Converts a string in base 16 to a span id.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::trace::SpanId;
    ///
    /// assert!(SpanId::from_hex("42").is_ok());
    /// assert!(SpanId::from_hex("58406520a0066491").is_ok());
    ///
    /// assert!(SpanId::from_hex("not_hex").is_err());
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, ParseIntError> {
        u64::from_str_radix(hex, 16).map(SpanId)
    }
}

impl From<[u8; 8]> for SpanId {
    fn from(bytes: [u8; 8]) -> Self {
        SpanId::from_bytes(bytes)
    }
}

impl fmt::Debug for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:016x}", self.0))
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:016x}", self.0))
    }
}

impl fmt::LowerHex for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// TraceState carries system-specific configuration data, represented as a list
/// of key-value pairs. TraceState allows multiple tracing systems to
/// participate in the same trace.
///
/// Please review the [W3C specification] for details on this field.
///
/// [W3C specification]: https://www.w3.org/TR/trace-context/#tracestate-header
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct TraceState(Option<VecDeque<(String, String)>>);

impl TraceState {
    /// Validates that the given `TraceState` list-member key is valid per the [W3 Spec].
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context/#key
    fn valid_key(key: &str) -> bool {
        if key.len() > 256 {
            return false;
        }

        let allowed_special = |b: u8| (b == b'_' || b == b'-' || b == b'*' || b == b'/');
        let mut vendor_start = None;
        for (i, &b) in key.as_bytes().iter().enumerate() {
            if !(b.is_ascii_lowercase() || b.is_ascii_digit() || allowed_special(b) || b == b'@') {
                return false;
            }

            if i == 0 && (!b.is_ascii_lowercase() && !b.is_ascii_digit()) {
                return false;
            } else if b == b'@' {
                if vendor_start.is_some() || i + 14 < key.len() {
                    return false;
                }
                vendor_start = Some(i);
            } else if let Some(start) = vendor_start {
                if i == start + 1 && !(b.is_ascii_lowercase() || b.is_ascii_digit()) {
                    return false;
                }
            }
        }

        true
    }

    /// Validates that the given `TraceState` list-member value is valid per the [W3 Spec].
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context/#value
    fn valid_value(value: &str) -> bool {
        if value.len() > 256 {
            return false;
        }

        !(value.contains(',') || value.contains('='))
    }

    /// Creates a new `TraceState` from the given key-value collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::trace::TraceState;
    ///
    /// let kvs = vec![("foo", "bar"), ("apple", "banana")];
    /// let trace_state = TraceState::from_key_value(kvs);
    ///
    /// assert!(trace_state.is_ok());
    /// assert_eq!(trace_state.unwrap().header(), String::from("foo=bar,apple=banana"))
    /// ```
    pub fn from_key_value<T, K, V>(trace_state: T) -> TraceResult<Self>
    where
        T: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        let ordered_data = trace_state
            .into_iter()
            .map(|(key, value)| {
                let (key, value) = (key.to_string(), value.to_string());
                if !TraceState::valid_key(key.as_str()) {
                    return Err(TraceStateError::Key(key));
                }
                if !TraceState::valid_value(value.as_str()) {
                    return Err(TraceStateError::Value(value));
                }

                Ok((key, value))
            })
            .collect::<Result<VecDeque<_>, TraceStateError>>()?;

        if ordered_data.is_empty() {
            Ok(TraceState(None))
        } else {
            Ok(TraceState(Some(ordered_data)))
        }
    }

    /// Retrieves a value for a given key from the `TraceState` if it exists.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.as_ref().and_then(|kvs| {
            kvs.iter().find_map(|item| {
                if item.0.as_str() == key {
                    Some(item.1.as_str())
                } else {
                    None
                }
            })
        })
    }

    /// Inserts the given key-value pair into the `TraceState`. If a value already exists for the
    /// given key, this updates the value and updates the value's position. If the key or value are
    /// invalid per the [W3 Spec] an `Err` is returned, else a new `TraceState` with the
    /// updated key/value is returned.
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context/#mutating-the-tracestate-field
    pub fn insert<K, V>(&self, key: K, value: V) -> TraceResult<TraceState>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let (key, value) = (key.into(), value.into());
        if !TraceState::valid_key(key.as_str()) {
            return Err(TraceStateError::Key(key).into());
        }
        if !TraceState::valid_value(value.as_str()) {
            return Err(TraceStateError::Value(value).into());
        }

        let mut trace_state = self.delete_from_deque(key.clone());
        let kvs = trace_state.0.get_or_insert(VecDeque::with_capacity(1));

        kvs.push_front((key, value));

        Ok(trace_state)
    }

    /// Removes the given key-value pair from the `TraceState`. If the key is invalid per the
    /// [W3 Spec] an `Err` is returned. Else, a new `TraceState`
    /// with the removed entry is returned.
    ///
    /// If the key is not in `TraceState`. The original `TraceState` will be cloned and returned.
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context/#mutating-the-tracestate-field
    pub fn delete<K: Into<String>>(&self, key: K) -> TraceResult<TraceState> {
        let key = key.into();
        if !TraceState::valid_key(key.as_str()) {
            return Err(TraceStateError::Key(key).into());
        }

        Ok(self.delete_from_deque(key))
    }

    /// Delete key from trace state's deque. The key MUST be valid
    fn delete_from_deque(&self, key: String) -> TraceState {
        let mut owned = self.clone();
        if let Some(kvs) = owned.0.as_mut() {
            if let Some(index) = kvs.iter().position(|x| *x.0 == *key) {
                kvs.remove(index);
            }
        }
        owned
    }

    /// Creates a new `TraceState` header string, delimiting each key and value with a `=` and each
    /// entry with a `,`.
    pub fn header(&self) -> String {
        self.header_delimited("=", ",")
    }

    /// Creates a new `TraceState` header string, with the given key/value delimiter and entry delimiter.
    pub fn header_delimited(&self, entry_delimiter: &str, list_delimiter: &str) -> String {
        self.0
            .as_ref()
            .map(|kvs| {
                kvs.iter()
                    .map(|(key, value)| format!("{}{}{}", key, entry_delimiter, value))
                    .collect::<Vec<String>>()
                    .join(list_delimiter)
            })
            .unwrap_or_default()
    }
}

impl FromStr for TraceState {
    type Err = TraceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let list_members: Vec<&str> = s.split_terminator(',').collect();
        let mut key_value_pairs: Vec<(String, String)> = Vec::with_capacity(list_members.len());

        for list_member in list_members {
            match list_member.find('=') {
                None => return Err(TraceStateError::List(list_member.to_string()).into()),
                Some(separator_index) => {
                    let (key, value) = list_member.split_at(separator_index);
                    key_value_pairs
                        .push((key.to_string(), value.trim_start_matches('=').to_string()));
                }
            }
        }

        TraceState::from_key_value(key_value_pairs)
    }
}

/// Error returned by `TraceState` operations.
#[derive(Error, Debug)]
#[non_exhaustive]
enum TraceStateError {
    /// The key is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context/#key> for requirement for keys.
    #[error("{0} is not a valid key in TraceState, see https://www.w3.org/TR/trace-context/#key for more details")]
    Key(String),

    /// The value is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context/#value> for requirement for values.
    #[error("{0} is not a valid value in TraceState, see https://www.w3.org/TR/trace-context/#value for more details")]
    Value(String),

    /// The list is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context/#list> for requirement for list members.
    #[error("{0} is not a valid list member in TraceState, see https://www.w3.org/TR/trace-context/#list for more details")]
    List(String),
}

impl From<TraceStateError> for TraceError {
    fn from(err: TraceStateError) -> Self {
        TraceError::Other(Box::new(err))
    }
}

/// Immutable portion of a [`Span`] which can be serialized and propagated.
///
/// This representation conforms to the [W3C TraceContext specification].
///
/// Spans that do not have the `sampled` flag set in their [`TraceFlags`] will
/// be ignored by most tracing tools.
///
/// [`Span`]: crate::trace::Span
/// [W3C TraceContext specification]: https://www.w3.org/TR/trace-context
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct SpanContext {
    trace_id: TraceId,
    span_id: SpanId,
    trace_flags: TraceFlags,
    is_remote: bool,
    trace_state: TraceState,
}

impl SpanContext {
    /// Create an invalid empty span context
    pub fn empty_context() -> Self {
        SpanContext::new(
            TraceId::INVALID,
            SpanId::INVALID,
            TraceFlags::default(),
            false,
            TraceState::default(),
        )
    }

    /// Construct a new `SpanContext`
    pub fn new(
        trace_id: TraceId,
        span_id: SpanId,
        trace_flags: TraceFlags,
        is_remote: bool,
        trace_state: TraceState,
    ) -> Self {
        SpanContext {
            trace_id,
            span_id,
            trace_flags,
            is_remote,
            trace_state,
        }
    }

    /// The [`TraceId`] for this span context.
    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    /// The [`SpanId`] for this span context.
    pub fn span_id(&self) -> SpanId {
        self.span_id
    }

    /// Returns details about the trace.
    ///
    /// Unlike `TraceState` values, these are present in all traces. The current
    /// version of the specification only supports a single flag [`TraceFlags::SAMPLED`].
    pub fn trace_flags(&self) -> TraceFlags {
        self.trace_flags
    }

    /// Returns `true` if the span context has a valid (non-zero) `trace_id` and a
    /// valid (non-zero) `span_id`.
    pub fn is_valid(&self) -> bool {
        self.trace_id != TraceId::INVALID && self.span_id != SpanId::INVALID
    }

    /// Returns `true` if the span context was propagated from a remote parent.
    pub fn is_remote(&self) -> bool {
        self.is_remote
    }

    /// Returns `true` if the `sampled` trace flag is set.
    ///
    /// Spans that are not sampled will be ignored by most tracing tools.
    pub fn is_sampled(&self) -> bool {
        self.trace_flags.is_sampled()
    }

    /// A reference to the span context's [`TraceState`].
    pub fn trace_state(&self) -> &TraceState {
        &self.trace_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    fn trace_id_test_data() -> Vec<(TraceId, &'static str, [u8; 16])> {
        vec![
            (TraceId(0), "00000000000000000000000000000000", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            (TraceId(42), "0000000000000000000000000000002a", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42]),
            (TraceId(126642714606581564793456114182061442190), "5f467fe7bf42676c05e20ba4a90e448e", [95, 70, 127, 231, 191, 66, 103, 108, 5, 226, 11, 164, 169, 14, 68, 142])
        ]
    }

    #[rustfmt::skip]
    fn span_id_test_data() -> Vec<(SpanId, &'static str, [u8; 8])> {
        vec![
            (SpanId(0), "0000000000000000", [0, 0, 0, 0, 0, 0, 0, 0]),
            (SpanId(42), "000000000000002a", [0, 0, 0, 0, 0, 0, 0, 42]),
            (SpanId(5508496025762705295), "4c721bf33e3caf8f", [76, 114, 27, 243, 62, 60, 175, 143])
        ]
    }

    #[rustfmt::skip]
    fn trace_state_test_data() -> Vec<(TraceState, &'static str, &'static str)> {
        vec![
            (TraceState::from_key_value(vec![("foo", "bar")]).unwrap(), "foo=bar", "foo"),
            (TraceState::from_key_value(vec![("foo", ""), ("apple", "banana")]).unwrap(), "foo=,apple=banana", "apple"),
            (TraceState::from_key_value(vec![("foo", "bar"), ("apple", "banana")]).unwrap(), "foo=bar,apple=banana", "apple"),
        ]
    }

    #[test]
    fn test_trace_id() {
        for test_case in trace_id_test_data() {
            assert_eq!(format!("{}", test_case.0), test_case.1);
            assert_eq!(format!("{:032x}", test_case.0), test_case.1);
            assert_eq!(test_case.0.to_bytes(), test_case.2);

            assert_eq!(test_case.0, TraceId::from_hex(test_case.1).unwrap());
            assert_eq!(test_case.0, TraceId::from_bytes(test_case.2));
        }
    }

    #[test]
    fn test_span_id() {
        for test_case in span_id_test_data() {
            assert_eq!(format!("{}", test_case.0), test_case.1);
            assert_eq!(format!("{:016x}", test_case.0), test_case.1);
            assert_eq!(test_case.0.to_bytes(), test_case.2);

            assert_eq!(test_case.0, SpanId::from_hex(test_case.1).unwrap());
            assert_eq!(test_case.0, SpanId::from_bytes(test_case.2));
        }
    }

    #[test]
    fn test_trace_state() {
        for test_case in trace_state_test_data() {
            assert_eq!(test_case.0.clone().header(), test_case.1);

            let new_key = format!("{}-{}", test_case.0.get(test_case.2).unwrap(), "test");

            let updated_trace_state = test_case.0.insert(test_case.2, new_key.clone());
            assert!(updated_trace_state.is_ok());
            let updated_trace_state = updated_trace_state.unwrap();

            let updated = format!("{}={}", test_case.2, new_key);

            let index = updated_trace_state.clone().header().find(&updated);

            assert!(index.is_some());
            assert_eq!(index.unwrap(), 0);

            let deleted_trace_state = updated_trace_state.delete(test_case.2.to_string());
            assert!(deleted_trace_state.is_ok());

            let deleted_trace_state = deleted_trace_state.unwrap();

            assert!(deleted_trace_state.get(test_case.2).is_none());
        }
    }

    #[test]
    fn test_trace_state_key() {
        let test_data: Vec<(&'static str, bool)> = vec![
            ("123", true),
            ("bar", true),
            ("foo@bar", true),
            ("foo@0123456789abcdef", false),
            ("foo@012345678", true),
            ("FOO@BAR", false),
            ("你好", false),
        ];

        for (key, expected) in test_data {
            assert_eq!(TraceState::valid_key(key), expected, "test key: {:?}", key);
        }
    }

    #[test]
    fn test_trace_state_insert() {
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();
        let inserted_trace_state = trace_state.insert("testkey", "testvalue").unwrap();
        assert!(trace_state.get("testkey").is_none()); // The original state doesn't change
        assert_eq!(inserted_trace_state.get("testkey").unwrap(), "testvalue"); //
    }
}
