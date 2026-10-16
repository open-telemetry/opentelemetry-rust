//! Primitives for sending name/value data across system boundaries.
//!
//! Baggage is used to annotate telemetry, adding context and information to
//! metrics, traces, and logs. It is a set of name/value pairs describing
//! user-defined properties. Each name in Baggage is associated with exactly one
//! value.
//!
//! Main types in this module are:
//!
//! * [`Baggage`]: A set of name/value pairs describing user-defined properties.
//! * [`BaggageExt`]: Extensions for managing `Baggage` in a [`Context`].
//!
//! Baggage can be sent between systems using a baggage propagator in
//! accordance with the [W3C Baggage] specification.
//!
//! Note: Baggage is not automatically added to any telemetry. Users have to
//! explicitly add baggage entries to telemetry items.
//!
//!
//! [W3C Baggage]: https://w3c.github.io/baggage
use crate::{Context, Key, KeyValue, StringValue};
use std::collections::hash_map::Entry;
use std::collections::{hash_map, HashMap};
use std::fmt;
use std::sync::OnceLock;

static DEFAULT_BAGGAGE: OnceLock<Baggage> = OnceLock::new();

const MAX_KEY_VALUE_PAIRS: usize = 64;

/// Returns the default baggage, ensuring it is initialized only once.
#[inline]
fn get_default_baggage() -> &'static Baggage {
    DEFAULT_BAGGAGE.get_or_init(Baggage::default)
}

/// A set of name/value pairs describing user-defined properties.
///
/// ### Baggage Names
///
/// * Any non-empty UTF-8 strings. Propagators may restrict which names they
///   transmit, e.g. the W3C baggage propagator only propagates names that are
///   valid tokens.
///
/// ### Baggage Values
///
/// * Arbitrary UTF-8 strings stored decoded. Percent-encoding for the wire is
///   handled by the propagator.
///
/// ### Baggage Value Metadata
///
/// Additional metadata can be added to values as an opaque [`BaggageMetadata`]
/// string. See its documentation for the requirements when propagating it.
///
/// ### Limits
///
/// * Maximum number of name/value pairs: `64`.
///
/// The W3C maximum serialized size of `8192` bytes depends on the wire
/// encoding and is enforced by the baggage propagator, not by this type.
///
/// <https://www.w3.org/TR/baggage/#limits>
#[derive(Debug, Default)]
pub struct Baggage {
    inner: HashMap<Key, (StringValue, BaggageMetadata)>,
}

impl Baggage {
    /// Creates an empty `Baggage`.
    pub fn new() -> Self {
        Baggage {
            inner: HashMap::default(),
        }
    }

    /// Returns a reference to the value associated with a given name
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::Baggage, StringValue};
    ///
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert("my-name", "my-value");
    ///
    /// assert_eq!(baggage.get("my-name"), Some(&StringValue::from("my-value")))
    /// ```
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&StringValue> {
        self.inner.get(key.as_ref()).map(|(value, _metadata)| value)
    }

    /// Returns a reference to the value and metadata associated with a given name
    ///
    /// # Examples
    /// ```
    /// use opentelemetry::{baggage::{Baggage, BaggageMetadata}, StringValue};
    ///
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert("my-name", "my-value");
    ///
    /// // By default, the metadata is empty
    /// assert_eq!(baggage.get_with_metadata("my-name"), Some(&(StringValue::from("my-value"), BaggageMetadata::from(""))))
    /// ```
    pub fn get_with_metadata<K: AsRef<str>>(
        &self,
        key: K,
    ) -> Option<&(StringValue, BaggageMetadata)> {
        self.inner.get(key.as_ref())
    }

    /// Inserts a name/value pair into the baggage.
    ///
    /// If the name was not present, [`None`] is returned. If the name was present,
    /// the value is updated, and the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::Baggage, StringValue};
    ///
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert("my-name", "my-value");
    ///
    /// assert_eq!(baggage.get("my-name"), Some(&StringValue::from("my-value")))
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<StringValue>
    where
        K: Into<Key>,
        V: Into<StringValue>,
    {
        self.insert_with_metadata(key, value, BaggageMetadata::default())
            .map(|pair| pair.0)
    }

    /// Inserts a name/value(+metadata) pair into the baggage.
    ///
    /// Same with `insert`, if the name was not present, [`None`] will be returned.
    /// If the name is present, the old value and metadata will be returned.
    ///
    /// The pair is not inserted if the name is empty or if the
    /// name is new and the baggage already holds the maximum number of
    /// entries (see [limits](https://w3c.github.io/baggage/#limits)).
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::{Baggage, BaggageMetadata}, StringValue};
    ///
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert_with_metadata("my-name", "my-value", "test");
    ///
    /// assert_eq!(baggage.get_with_metadata("my-name"), Some(&(StringValue::from("my-value"), BaggageMetadata::from("test"))))
    /// ```
    pub fn insert_with_metadata<K, V, S>(
        &mut self,
        key: K,
        value: V,
        metadata: S,
    ) -> Option<(StringValue, BaggageMetadata)>
    where
        K: Into<Key>,
        V: Into<StringValue>,
        S: Into<BaggageMetadata>,
    {
        let (key, value, metadata) = (key.into(), value.into(), metadata.into());
        let entries_count = self.inner.len();
        match self.inner.entry(key) {
            Entry::Occupied(mut occupied_entry) => Some(occupied_entry.insert((value, metadata))),
            Entry::Vacant(vacant_entry) => {
                if !vacant_entry.key().as_str().is_empty() && entries_count < MAX_KEY_VALUE_PAIRS {
                    vacant_entry.insert((value, metadata));
                }
                None
            }
        }
    }

    /// Removes a name from the baggage, returning the value
    /// corresponding to the name if the pair was previously in the map.
    pub fn remove<K: AsRef<str>>(&mut self, key: K) -> Option<(StringValue, BaggageMetadata)> {
        self.inner.remove(key.as_ref())
    }

    /// Returns the number of attributes for this baggage
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the baggage contains no items.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets an iterator over the baggage items, in any order.
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

/// An iterator over the entries of a [`Baggage`].
#[derive(Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, Key, (StringValue, BaggageMetadata)>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Key, &'a (StringValue, BaggageMetadata));

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Baggage {
    type Item = (&'a Key, &'a (StringValue, BaggageMetadata));
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}

impl FromIterator<(Key, (StringValue, BaggageMetadata))> for Baggage {
    fn from_iter<I: IntoIterator<Item = (Key, (StringValue, BaggageMetadata))>>(iter: I) -> Self {
        let mut baggage = Baggage::default();
        for (key, (value, metadata)) in iter.into_iter() {
            baggage.insert_with_metadata(key, value, metadata);
        }
        baggage
    }
}

impl FromIterator<KeyValue> for Baggage {
    fn from_iter<I: IntoIterator<Item = KeyValue>>(iter: I) -> Self {
        let mut baggage = Baggage::default();
        for kv in iter.into_iter() {
            baggage.insert(kv.key, kv.value);
        }
        baggage
    }
}

impl FromIterator<KeyValueMetadata> for Baggage {
    fn from_iter<I: IntoIterator<Item = KeyValueMetadata>>(iter: I) -> Self {
        let mut baggage = Baggage::default();
        for kvm in iter.into_iter() {
            baggage.insert_with_metadata(kvm.key, kvm.value, kvm.metadata);
        }
        baggage
    }
}

impl<I> From<I> for Baggage
where
    I: IntoIterator,
    I::Item: Into<KeyValueMetadata>,
{
    fn from(value: I) -> Self {
        value.into_iter().map(Into::into).collect()
    }
}

/// Methods for sorting and retrieving baggage data in a context.
pub trait BaggageExt {
    /// Returns a clone of the given context with the included name/value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::{Baggage, BaggageExt}, Context, KeyValue, StringValue};
    ///
    /// // Explicit `Baggage` creation
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert("my-name", "my-value");
    ///
    /// let cx = Context::map_current(|cx| {
    ///     cx.with_baggage(baggage)
    /// });
    ///
    /// // Passing an iterator
    /// let cx = Context::map_current(|cx| {
    ///     cx.with_baggage([KeyValue::new("my-name", "my-value")])
    /// });
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&StringValue::from("my-value")),
    /// )
    /// ```
    fn with_baggage<T: Into<Baggage>>(&self, baggage: T) -> Self;

    /// Returns a clone of the current context with the included name/value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::{Baggage, BaggageExt}, Context, StringValue};
    ///
    /// let mut baggage = Baggage::new();
    /// let _ = baggage.insert("my-name", "my-value");
    ///
    /// let cx = Context::current_with_baggage(baggage);
    ///
    /// assert_eq!(
    ///     cx.baggage().get("my-name"),
    ///     Some(&StringValue::from("my-value")),
    /// )
    /// ```
    fn current_with_baggage<T: Into<Baggage>>(baggage: T) -> Self;

    /// Returns a clone of the given context with no baggage.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{baggage::BaggageExt, Context};
    ///
    /// let cx = Context::map_current(|cx| cx.with_cleared_baggage());
    ///
    /// assert_eq!(cx.baggage().len(), 0);
    /// ```
    fn with_cleared_baggage(&self) -> Self;

    /// Returns a reference to this context's baggage, or the default
    /// empty baggage if none has been set.
    fn baggage(&self) -> &Baggage;
}

/// Solely used to store `Baggage` in the `Context` without allowing direct access
#[derive(Debug)]
struct BaggageContextValue(Baggage);

impl BaggageExt for Context {
    fn with_baggage<T: Into<Baggage>>(&self, baggage: T) -> Self {
        self.with_value(BaggageContextValue(baggage.into()))
    }

    fn current_with_baggage<T: Into<Baggage>>(baggage: T) -> Self {
        Context::map_current(|cx| cx.with_baggage(baggage))
    }

    fn with_cleared_baggage(&self) -> Self {
        self.with_baggage(Baggage::new())
    }

    fn baggage(&self) -> &Baggage {
        self.get::<BaggageContextValue>()
            .map_or(get_default_baggage(), |b| &b.0)
    }
}

/// An optional property set that can be added to [`Baggage`] values.
///
/// `BaggageMetadata` is an opaque string that is stored exactly as given. It
/// is not parsed, validated or encoded by the API.
///
/// When baggage is propagated with the [W3C Baggage] format, the metadata is
/// written verbatim after the value, so it must already be valid W3C
/// `property` syntax: a semi-colon `;` delimited list of names and/or
/// name/value pairs, e.g. `k1=v1;k2;k3=v3`, where names are tokens and
/// values are percent-encoded. Entries whose metadata is not valid are
/// dropped by the propagator rather than sent malformed.
///
/// Metadata extracted by the propagator is likewise kept in its encoded wire
/// form.
///
/// [W3C Baggage]: https://www.w3.org/TR/baggage/#property
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Default)]
pub struct BaggageMetadata(String);

impl BaggageMetadata {
    /// Return underlying string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for BaggageMetadata {
    fn from(s: String) -> BaggageMetadata {
        BaggageMetadata(s)
    }
}

impl From<&str> for BaggageMetadata {
    fn from(s: &str) -> Self {
        BaggageMetadata(s.to_string())
    }
}

impl fmt::Display for BaggageMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(write!(f, "{}", self.as_str())?)
    }
}

/// [`Baggage`] name/value pairs with their associated metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValueMetadata {
    /// Dimension or event key
    pub(crate) key: Key,
    /// Dimension or event value
    pub(crate) value: StringValue,
    /// Metadata associate with this key value pair
    pub(crate) metadata: BaggageMetadata,
}

impl KeyValueMetadata {
    /// Create a new `KeyValue` pair with metadata
    pub fn new<K, V, S>(key: K, value: V, metadata: S) -> Self
    where
        K: Into<Key>,
        V: Into<StringValue>,
        S: Into<BaggageMetadata>,
    {
        KeyValueMetadata {
            key: key.into(),
            value: value.into(),
            metadata: metadata.into(),
        }
    }
}

impl From<KeyValue> for KeyValueMetadata {
    fn from(kv: KeyValue) -> Self {
        KeyValueMetadata {
            key: kv.key,
            value: kv.value.into(),
            metadata: BaggageMetadata::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StringValue;

    use super::*;

    #[test]
    fn insert_non_ascii_key() {
        let mut baggage = Baggage::new();
        baggage.insert("🚫", "not ascii key");
        assert_eq!(baggage.len(), 1, "non-ascii names are allowed");
    }

    #[test]
    fn insert_too_much_baggage() {
        // too many key pairs
        let over_limit = MAX_KEY_VALUE_PAIRS + 1;
        let mut data = Vec::with_capacity(over_limit);
        for i in 0..over_limit {
            data.push(KeyValue::new(format!("key{i}"), format!("key{i}")))
        }
        let baggage = data.into_iter().collect::<Baggage>();
        assert_eq!(baggage.len(), MAX_KEY_VALUE_PAIRS)
    }

    #[test]
    fn insert_does_not_limit_size() {
        let large: String = (0..10_000).map(|_| 'x').collect();
        let mut baggage = Baggage::new();
        baggage.insert("a", large.clone());
        baggage.insert("b", large);
        assert_eq!(baggage.len(), 2);
    }

    #[test]
    fn replace_existing_key_at_capacity() {
        let mut b = (0..MAX_KEY_VALUE_PAIRS)
            .map(|i| KeyValue::new(format!("key{i}"), "v"))
            .collect::<Baggage>();
        assert_eq!(b.len(), MAX_KEY_VALUE_PAIRS);

        assert!(b.insert("other", "v").is_none());
        assert!(b.get("other").is_none());

        assert_eq!(b.insert("key0", "w"), Some(StringValue::from("v")));
        assert_eq!(b.get("key0"), Some(&StringValue::from("w")));
    }

    #[test]
    fn remove_frees_capacity() {
        let mut b = (0..MAX_KEY_VALUE_PAIRS)
            .map(|i| KeyValue::new(format!("key{i}"), "v"))
            .collect::<Baggage>();

        for _ in 0..3 {
            assert!(b.remove("key0").is_some());
            b.insert("key0", "v");
            assert_eq!(b.get("key0"), Some(&StringValue::from("v")));
            assert_eq!(b.len(), MAX_KEY_VALUE_PAIRS);
        }
    }

    #[test]
    fn metadata_is_stored_verbatim() {
        assert_eq!(BaggageMetadata::from(" p=1 ").as_str(), " p=1 ");
        assert_eq!(
            BaggageMetadata::from(String::from("\tp\t")).as_str(),
            "\tp\t"
        );
    }

    #[test]
    fn test_crud_operations() {
        let mut baggage = Baggage::default();
        assert!(baggage.is_empty());

        // create
        baggage.insert("foo", "1");
        assert_eq!(baggage.len(), 1);

        // get
        assert_eq!(baggage.get("foo"), Some(&StringValue::from("1")));

        // update
        baggage.insert("foo", "2");
        assert_eq!(baggage.get("foo"), Some(&StringValue::from("2")));

        // delete
        baggage.remove("foo");
        assert!(baggage.is_empty());
    }

    #[test]
    fn test_insert_key_validation() {
        let mut baggage = Baggage::default();

        // empty names are rejected
        baggage.insert("", "1");
        assert!(baggage.is_empty());

        // any other UTF-8 string is allowed, propagators may restrict it
        baggage.insert("Grüße", "1");
        baggage.insert("(example)", "1");
        baggage.insert("a b", "1");
        assert_eq!(baggage.len(), 3);
    }

    #[test]
    fn test_context_clear_baggage() {
        let ctx = Context::new();
        let ctx = ctx.with_baggage([KeyValue::new("foo", 1)]);
        let _guard = ctx.attach();

        {
            let ctx = Context::current();
            let baggage = ctx.baggage();
            // At this point baggage should still contain the inital value.
            assert_eq!(baggage.len(), 1);

            // Baggage gets cleared.
            let ctx = ctx.with_cleared_baggage();
            let _guard = ctx.attach();
            {
                let ctx = Context::current();
                let baggage = ctx.baggage();
                // Baggage should contain no entries.
                assert_eq!(baggage.len(), 0);
            }
        }
    }
}
