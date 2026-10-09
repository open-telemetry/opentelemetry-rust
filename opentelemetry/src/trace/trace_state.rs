use std::collections::VecDeque;
use std::str::FromStr;
use thiserror::Error;

/// TraceState carries system-specific configuration data, represented as a list
/// of key-value pairs. TraceState allows multiple tracing systems to
/// participate in the same trace.
///
/// Please review the [W3C specification] for details on this field.
///
/// [W3C specification]: https://www.w3.org/TR/trace-context-2/#tracestate-header
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct TraceState(VecDeque<(String, String)>);

/// The maximum number of list-members a `TraceState` may hold.
const MAX_LIST_MEMBERS: usize = 32;

/// The maximum length of a list-member key or value.
const MAX_KEY_VALUE_LEN: usize = 256;

impl TraceState {
    /// The default `TraceState`, as a constant
    pub const NONE: TraceState = TraceState(VecDeque::new());

    /// Validates that the given `TraceState` list-member key is valid per the [W3 Spec].
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context-2/#key
    /// 1-256 char, starting with ASCII lowercase letter or digit. The rest may also
    /// include  `_`, `-`, `*`, `/`, `@` 
    fn valid_key(key: &str) -> bool {
        let bytes = key.as_bytes();
        match bytes.split_first() {
            Some((first, rest)) if bytes.len() <= MAX_KEY_VALUE_LEN => {
                (first.is_ascii_lowercase() || first.is_ascii_digit())
                    && rest.iter().all(|&b| {
                        b.is_ascii_lowercase()
                            || b.is_ascii_digit()
                            || matches!(b, b'_' | b'-' | b'*' | b'/' | b'@')
                    })
            }
            _ => false,
        }
    }

    /// Validates that the given `TraceState` list-member value is valid per the [W3 Spec].
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context-2/#value
    fn valid_value(value: &str) -> bool {
        let bytes = value.as_bytes();
        match bytes.last() {
            Some(&last) if bytes.len() <= MAX_KEY_VALUE_LEN => {
                last != b' '
                    && bytes
                        .iter()
                        .all(|&b| matches!(b, 0x20..=0x7E) && b != b',' && b != b'=')
            }
            _ => false,
        }
    }

    /// Validates the given key and value, returning an error naming whichever is invalid.
    fn validate(key: String, value: String) -> TraceStateResult<(String, String)> {
        if !TraceState::valid_key(&key) {
            return Err(TraceStateError::Key(key));
        }
        if !TraceState::valid_value(&value) {
            return Err(TraceStateError::Value(value));
        }
        Ok((key, value))
    }

    /// Builds a `TraceState` from already validated pairs, rejecting duplicate keys.
    fn from_validated_pairs<I>(pairs: I) -> TraceStateResult<Self>
    where
        I: IntoIterator<Item = TraceStateResult<(String, String)>>,
    {
        let mut ordered_data: VecDeque<(String, String)> = VecDeque::new();
        for pair in pairs {
            let (key, value) = pair?;
            if ordered_data.iter().any(|(existing, _)| *existing == key) {
                return Err(TraceStateError::DuplicateKey(key));
            }
            ordered_data.push_back((key, value));
        }

        Ok(TraceState(ordered_data))
    }

    /// Creates a new `TraceState` from the given key-value collection, keeping at most the 32
    /// list-members the [W3C specification] allows.
    ///
    /// Returns an `Err` if any of the kept keys or values is invalid, or if a key appears more
    /// than once.
    ///
    /// [W3C specification]: https://www.w3.org/TR/trace-context-2/#tracestate-header-field-values
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::trace::TraceState;
    ///
    /// let kvs = vec![("foo", "bar"), ("apple", "banana")];
    /// let trace_state = TraceState::from_key_value(kvs);
    ///
    /// assert!(trace_state.is_ok());
    /// assert_eq!(trace_state.unwrap().header(), String::from("foo=bar,apple=banana"))
    /// ```
    pub fn from_key_value<T, K, V>(trace_state: T) -> TraceStateResult<Self>
    where
        T: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        TraceState::from_validated_pairs(
            trace_state
                .into_iter()
                .take(MAX_LIST_MEMBERS)
                .map(|(key, value)| TraceState::validate(key.to_string(), value.to_string())),
        )
    }

    /// Retrieves a value for a given key from the `TraceState` if it exists.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, value)| value.as_str())
    }

    /// Inserts the given key-value pair into the `TraceState`. If a value already exists for the
    /// given key, this updates the value and updates the value's position. If the key or value are
    /// invalid per the [W3 Spec] an `Err` is returned, else a new `TraceState` with the
    /// updated key/value is returned.
    ///
    /// A `TraceState` holds at most the 32 list-members the [W3 Spec] allows. Inserting into a full
    /// `TraceState` keeps the inserted pair and drops the last list-member.
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context-2/#mutating-the-tracestate-field
    pub fn insert<K, V>(&self, key: K, value: V) -> TraceStateResult<TraceState>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let (key, value) = TraceState::validate(key.into(), value.into())?;

        let mut trace_state = self.delete_from_deque(&key);
        trace_state.0.push_front((key, value));
        trace_state.0.truncate(MAX_LIST_MEMBERS);

        Ok(trace_state)
    }

    /// Removes the given key-value pair from the `TraceState`. If the key is invalid per the
    /// [W3 Spec] an `Err` is returned. Else, a new `TraceState`
    /// with the removed entry is returned.
    ///
    /// If the key is not in `TraceState`. The original `TraceState` will be cloned and returned.
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context-2/#mutating-the-tracestate-field
    pub fn delete<K: Into<String>>(&self, key: K) -> TraceStateResult<TraceState> {
        let key = key.into();
        if !TraceState::valid_key(key.as_str()) {
            return Err(TraceStateError::Key(key));
        }

        Ok(self.delete_from_deque(&key))
    }

    /// Delete every entry with the given key from the trace state's deque. The key MUST be valid
    fn delete_from_deque(&self, key: &str) -> TraceState {
        let mut owned = self.clone();
        owned.0.retain(|(k, _)| k != key);
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
            .iter()
            .map(|(key, value)| format!("{key}{entry_delimiter}{value}"))
            .collect::<Vec<String>>()
            .join(list_delimiter)
    }
}

impl FromStr for TraceState {
    type Err = TraceStateError;

    /// Parses a `tracestate` header value per the [W3 Spec].
    ///
    /// Empty and whitespace only list members are ignored and do not count towards the limit of
    /// 32 list members. Only the first 32 list members are kept, later ones are dropped without
    /// being validated. An `Err` is returned if any kept list member is malformed, has an invalid
    /// key or value or repeats a key.
    ///
    /// [W3 Spec]: https://www.w3.org/TR/trace-context-2/#tracestate-header-field-values
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TraceState::from_validated_pairs(
            s.split(',')
                // W3C OWS around list members consists of spaces and horizontal tabs.
                .map(|list_member| list_member.trim_matches([' ', '\t']))
                .filter(|list_member| !list_member.is_empty())
                .take(MAX_LIST_MEMBERS)
                .map(|list_member| match list_member.split_once('=') {
                    Some((key, value)) => TraceState::validate(key.to_string(), value.to_string()),
                    None => Err(TraceStateError::List(list_member.to_string())),
                }),
        )
    }
}

/// Iterator over TraceState key-value pairs as (&str, &str)
#[derive(Debug)]
pub struct TraceStateIter<'a> {
    inner: std::collections::vec_deque::Iter<'a, (String, String)>,
}

impl<'a> Iterator for TraceStateIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for TraceStateIter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> IntoIterator for &'a TraceState {
    type Item = (&'a str, &'a str);
    type IntoIter = TraceStateIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TraceStateIter {
            inner: self.0.iter(),
        }
    }
}

/// A specialized `Result` type for trace state operations.
type TraceStateResult<T> = Result<T, TraceStateError>;

/// Error returned by `TraceState` operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TraceStateError {
    /// The key is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context-2/#key> for requirement for keys.
    #[error("{0} is not a valid key in TraceState, see https://www.w3.org/TR/trace-context-2/#key for more details")]
    Key(String),

    /// The value is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context-2/#value> for requirement for values.
    #[error("{0} is not a valid value in TraceState, see https://www.w3.org/TR/trace-context-2/#value for more details")]
    Value(String),

    /// The list is invalid.
    ///
    /// See <https://www.w3.org/TR/trace-context-2/#list> for requirement for list members.
    #[error("{0} is not a valid list member in TraceState, see https://www.w3.org/TR/trace-context-2/#list for more details")]
    List(String),

    /// The key appears more than once.
    ///
    /// See <https://www.w3.org/TR/trace-context-2/#combined-header-value> for requirement for
    /// unique keys.
    #[error("{0} appears more than once in TraceState, see https://www.w3.org/TR/trace-context-2/#combined-header-value for more details")]
    DuplicateKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    fn trace_state_test_data() -> Vec<(TraceState, &'static str, &'static str)> {
        vec![
            (TraceState::from_key_value(vec![("foo", "bar")]).unwrap(), "foo=bar", "foo"),
            (TraceState::from_key_value(vec![("foo", " x"), ("apple", "banana")]).unwrap(), "foo= x,apple=banana", "apple"),
            (TraceState::from_key_value(vec![("foo", "bar"), ("apple", "banana")]).unwrap(), "foo=bar,apple=banana", "apple"),
        ]
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
        let test_data: Vec<(String, bool)> = vec![
            ("123".into(), true),
            ("bar".into(), true),
            ("foo@bar".into(), true),
            ("foo@0123456789abcdef".into(), true),
            ("foo@012345678".into(), true),
            ("a@b@c".into(), true),
            ("a_-*/@".into(), true),
            ("0".into(), true),
            ("a".repeat(256), true),
            ("a".repeat(257), false),
            ("".into(), false),
            ("_a".into(), false),
            ("@a".into(), false),
            ("-a".into(), false),
            ("Abc".into(), false),
            ("aBc".into(), false),
            ("FOO@BAR".into(), false),
            ("a b".into(), false),
            ("a ".into(), false),
            ("a=b".into(), false),
            ("a,b".into(), false),
            ("a.b".into(), false),
            ("你好".into(), false),
        ];

        for (key, expected) in test_data {
            assert_eq!(TraceState::valid_key(&key), expected, "test key: {key:?}");
        }
    }

    #[test]
    fn test_trace_state_value() {
        let test_data: Vec<(String, bool)> = vec![
            ("bar".into(), true),
            ("x".into(), true),
            (" x".into(), true),
            ("a b".into(), true),
            ("!\"#$%&'()*+-./:;<>?@[\\]^_`{|}~".into(), true),
            ("v".repeat(256), true),
            ("v".repeat(257), false),
            ("".into(), false),
            (" ".into(), false),
            ("x ".into(), false),
            ("a=b".into(), false),
            ("=".into(), false),
            ("a,b".into(), false),
            ("x\ty".into(), false),
            ("x\n".into(), false),
            ("x\u{7f}".into(), false),
            ("é".into(), false),
        ];

        for (value, expected) in test_data {
            assert_eq!(
                TraceState::valid_value(&value),
                expected,
                "test value: {value:?}"
            );
        }
    }

    #[test]
    fn test_trace_state_insert() {
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();
        let inserted_trace_state = trace_state.insert("testkey", "testvalue").unwrap();
        assert!(trace_state.get("testkey").is_none()); // The original state doesn't change
        assert_eq!(inserted_trace_state.get("testkey").unwrap(), "testvalue"); //
    }

    #[test]
    fn test_trace_state_insert_rejects_invalid_key_and_value() {
        let trace_state = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();

        assert!(matches!(
            trace_state.insert("", "x"),
            Err(TraceStateError::Key(_))
        ));
        assert!(matches!(
            trace_state.insert("a", ""),
            Err(TraceStateError::Value(_))
        ));
        assert!(matches!(
            trace_state.insert("a", "x "),
            Err(TraceStateError::Value(_))
        ));
        assert!(matches!(
            trace_state.insert("a", "x\ty"),
            Err(TraceStateError::Value(_))
        ));
        assert!(matches!(
            trace_state.insert("a", "x=y"),
            Err(TraceStateError::Value(_))
        ));
    }

    #[test]
    fn test_trace_state_insert_existing_key_moves_it_to_the_front() {
        let trace_state =
            TraceState::from_str("congo=congosFirstPosition,rojo=rojosFirstPosition").unwrap();
        let updated = trace_state.insert("congo", "congosSecondPosition").unwrap();

        assert_eq!(
            updated.header(),
            "congo=congosSecondPosition,rojo=rojosFirstPosition"
        );

        let updated = updated.insert("rojo", "rojosSecondPosition").unwrap();
        assert_eq!(
            updated.header(),
            "rojo=rojosSecondPosition,congo=congosSecondPosition"
        );
    }

    #[test]
    fn test_trace_state_delete() {
        let trace_state = TraceState::from_str("foo=1,bar=2,baz=3").unwrap();

        assert_eq!(trace_state.delete("bar").unwrap().header(), "foo=1,baz=3");
        assert_eq!(trace_state.delete("missing").unwrap(), trace_state);
        assert!(matches!(
            trace_state.delete("Bad"),
            Err(TraceStateError::Key(_))
        ));

        let emptied = TraceState::from_str("foo=1")
            .unwrap()
            .delete("foo")
            .unwrap();
        assert_eq!(emptied, TraceState::NONE);
    }

    #[test]
    fn test_tracestate_iter_empty() {
        let ts = TraceState::NONE;
        let mut iter = ts.into_iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn test_tracestate_iter_single() {
        let ts = TraceState::from_key_value(vec![("foo", "bar")]).unwrap();
        let mut iter = ts.into_iter();
        assert_eq!(iter.next(), Some(("foo", "bar")));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_tracestate_iter_multiple() {
        let ts = TraceState::from_key_value(vec![("foo", "bar"), ("apple", "banana")]).unwrap();
        let mut iter = ts.into_iter();
        assert_eq!(iter.next(), Some(("foo", "bar")));
        assert_eq!(iter.next(), Some(("apple", "banana")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_tracestate_iter_size_hint_and_len() {
        let ts = TraceState::from_key_value(vec![("foo", "bar"), ("apple", "banana")]).unwrap();
        let iter = ts.into_iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.len(), 2);
    }

    #[test]
    fn test_tracestate_from_str_keeps_at_most_32_list_members() {
        let header = (0..64)
            .map(|i| format!("key{i}=value{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let trace_state = TraceState::from_str(&header).unwrap();

        assert_eq!(trace_state.into_iter().count(), 32);
        assert_eq!(trace_state.get("key0"), Some("value0"));
        assert_eq!(trace_state.get("key31"), Some("value31"));
        assert_eq!(trace_state.get("key32"), None);

        let expected_header = (0..32)
            .map(|i| format!("key{i}=value{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(trace_state.header(), expected_header);
    }

    #[test]
    fn test_tracestate_from_str_does_not_count_empty_members_towards_the_limit() {
        let mut members: Vec<String> = vec![String::new(); 40];
        members.extend((0..32).map(|i| format!("key{i}=value{i}")));
        let trace_state = TraceState::from_str(&members.join(", ")).unwrap();

        assert_eq!(trace_state.into_iter().count(), 32);
        assert_eq!(trace_state.get("key31"), Some("value31"));
    }

    #[test]
    fn test_tracestate_from_str_allows_optional_whitespace_around_members() {
        for header in [
            "rojo=1, congo=2",
            " rojo=1,\tcongo=2 \t",
            "rojo=1 \t, congo=2",
        ] {
            let trace_state = TraceState::from_str(header).unwrap();

            assert_eq!(trace_state.header(), "rojo=1,congo=2", "{header:?}");
        }
    }

    #[test]
    fn test_tracestate_from_str_preserves_leading_spaces_in_values() {
        let trace_state = TraceState::from_str("rojo= 1 , congo=  2\t").unwrap();

        assert_eq!(trace_state.get("rojo"), Some(" 1"));
        assert_eq!(trace_state.get("congo"), Some("  2"));
    }

    #[test]
    fn test_tracestate_from_str_ignores_empty_members() {
        let trace_state = TraceState::from_str("rojo=1, ,\t,congo=2").unwrap();

        assert_eq!(trace_state.get("rojo"), Some("1"));
        assert_eq!(trace_state.get("congo"), Some("2"));
        assert_eq!(trace_state.into_iter().count(), 2);

        assert_eq!(TraceState::from_str("").unwrap(), TraceState::NONE);
        assert_eq!(TraceState::from_str(" \t ").unwrap(), TraceState::NONE);
        assert_eq!(TraceState::from_str(",, ,").unwrap(), TraceState::NONE);
    }

    #[test]
    fn test_tracestate_from_str_rejects_invalid_members() {
        let test_data = [
            ("=x", "empty key"),
            ("a=", "empty value"),
            ("a= ", "whitespace-only value"),
            ("a=x\ty", "tab in value"),
            ("a=x\u{7f}", "DEL in value"),
            ("a==b", "repeated equals sign"),
            ("a=b=c", "equals sign in value"),
            ("a =1", "space before equals sign"),
            ("A=1", "upper case key"),
            ("_a=1", "key starting with underscore"),
            ("nokeyvalue", "missing equals sign"),
            ("foo=bar,=x", "empty key after valid member"),
            ("foo=bar,a=", "empty value after valid member"),
            ("foo=bar;baz=qux", "wrong list delimiter"),
        ];

        for (header, reason) in test_data {
            assert!(
                TraceState::from_str(header).is_err(),
                "{reason}: {header:?}"
            );
        }
    }

    #[test]
    fn test_tracestate_from_str_rejects_duplicate_keys() {
        for header in [
            "congo=1,rojo=2,congo=3",
            "congo=1,congo=1",
            "congo=1, ,congo=2",
        ] {
            assert!(
                matches!(
                    TraceState::from_str(header),
                    Err(TraceStateError::DuplicateKey(key)) if key == "congo"
                ),
                "{header:?}"
            );
        }
    }

    #[test]
    fn test_tracestate_from_str_ignores_invalid_members_beyond_the_limit() {
        let mut members: Vec<String> = (0..32).map(|i| format!("key{i}=value{i}")).collect();
        members.push("invalid-no-separator".to_string());
        members.push("key0=duplicate".to_string());
        let trace_state = TraceState::from_str(&members.join(",")).unwrap();

        assert_eq!(trace_state.into_iter().count(), 32);
        assert_eq!(trace_state.get("key0"), Some("value0"));
    }

    #[test]
    fn test_tracestate_from_str_rejects_invalid_members_within_the_limit() {
        let mut members: Vec<String> = (0..31).map(|i| format!("key{i}=value{i}")).collect();
        members.push("invalid-no-separator".to_string());

        assert!(TraceState::from_str(&members.join(",")).is_err());
    }

    #[test]
    fn test_tracestate_from_key_value_rejects_invalid_pairs() {
        assert!(matches!(
            TraceState::from_key_value(vec![("", "x")]),
            Err(TraceStateError::Key(_))
        ));
        assert!(matches!(
            TraceState::from_key_value(vec![("foo", "")]),
            Err(TraceStateError::Value(_))
        ));
        assert!(matches!(
            TraceState::from_key_value(vec![("foo", "x ")]),
            Err(TraceStateError::Value(_))
        ));
        assert!(matches!(
            TraceState::from_key_value(vec![("foo", "1"), ("bar", "2"), ("foo", "3")]),
            Err(TraceStateError::DuplicateKey(key)) if key == "foo"
        ));
    }

    #[test]
    fn test_tracestate_from_key_value_keeps_at_most_32_list_members() {
        let kvs: Vec<(String, String)> = (0..64)
            .map(|i| (format!("key{i}"), format!("value{i}")))
            .collect();
        let trace_state = TraceState::from_key_value(kvs).unwrap();

        assert_eq!(trace_state.into_iter().count(), 32);
        assert_eq!(trace_state.get("key0"), Some("value0"));
        assert_eq!(trace_state.get("key32"), None);
    }

    #[test]
    fn test_tracestate_insert_keeps_at_most_32_list_members() {
        let mut trace_state = TraceState::default();
        for i in 0..64 {
            trace_state = trace_state
                .insert(format!("key{i}"), format!("value{i}"))
                .unwrap();
        }

        assert_eq!(trace_state.into_iter().count(), 32);
        assert_eq!(trace_state.get("key63"), Some("value63"));
        assert_eq!(trace_state.get("key32"), Some("value32"));
        assert_eq!(trace_state.get("key31"), None);
    }

    #[test]
    fn test_tracestate_insert_of_an_existing_key_evicts_nothing() {
        let mut trace_state = TraceState::default();
        for i in 0..32 {
            trace_state = trace_state
                .insert(format!("key{i}"), format!("value{i}"))
                .unwrap();
        }
        let updated = trace_state.insert("key20", "updated").unwrap();

        assert_eq!(updated.into_iter().count(), 32);
        assert_eq!(updated.get("key20"), Some("updated"));
        assert_eq!(updated.get("key0"), Some("value0"));
    }
}
