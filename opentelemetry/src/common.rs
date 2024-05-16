use std::borrow::Cow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::{fmt, hash};

/// The key part of attribute [KeyValue] pairs.
///
/// See the [attribute naming] spec for guidelines.
///
/// [attribute naming]: https://github.com/open-telemetry/semantic-conventions/blob/main/docs/general/attribute-naming.md
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(OtelString);

impl Key {
    /// Create a new `Key`.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Key;
    /// use std::sync::Arc;
    ///
    /// let key1 = Key::new("my_static_str");
    /// let key2 = Key::new(String::from("my_owned_string"));
    /// let key3 = Key::new(Arc::from("my_ref_counted_str"));
    /// ```
    pub fn new(value: impl Into<Key>) -> Self {
        value.into()
    }

    /// Create a new const `Key`.
    pub const fn from_static_str(value: &'static str) -> Self {
        Key(OtelString::Static(value))
    }

    /// Create a `KeyValue` pair for `bool` values.
    pub fn bool<T: Into<bool>>(self, value: T) -> KeyValue {
        KeyValue {
            key: self,
            value: Value::Bool(value.into()),
        }
    }

    /// Create a `KeyValue` pair for `i64` values.
    pub fn i64(self, value: i64) -> KeyValue {
        KeyValue {
            key: self,
            value: Value::I64(value),
        }
    }

    /// Create a `KeyValue` pair for `f64` values.
    pub fn f64(self, value: f64) -> KeyValue {
        KeyValue {
            key: self,
            value: Value::F64(value),
        }
    }

    /// Create a `KeyValue` pair for string-like values.
    pub fn string(self, value: impl Into<StringValue>) -> KeyValue {
        KeyValue {
            key: self,
            value: Value::String(value.into()),
        }
    }

    /// Create a `KeyValue` pair for arrays.
    pub fn array<T: Into<Array>>(self, value: T) -> KeyValue {
        KeyValue {
            key: self,
            value: Value::Array(value.into()),
        }
    }

    /// Returns a reference to the underlying key name
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&'static str> for Key {
    /// Convert a `&str` to a `Key`.
    fn from(key_str: &'static str) -> Self {
        Key(OtelString::Static(key_str))
    }
}

impl From<String> for Key {
    /// Convert a `String` to a `Key`.
    fn from(string: String) -> Self {
        Key(OtelString::Owned(string.into_boxed_str()))
    }
}

impl From<Arc<str>> for Key {
    /// Convert a `String` to a `Key`.
    fn from(string: Arc<str>) -> Self {
        Key(OtelString::RefCounted(string))
    }
}

impl From<Cow<'static, str>> for Key {
    /// Convert a `Cow<'static, str>` to a `Key`
    fn from(string: Cow<'static, str>) -> Self {
        match string {
            Cow::Borrowed(s) => Key(OtelString::Static(s)),
            Cow::Owned(s) => Key(OtelString::Owned(s.into_boxed_str())),
        }
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl From<Key> for String {
    fn from(key: Key) -> Self {
        match key.0 {
            OtelString::Owned(s) => s.to_string(),
            OtelString::Static(s) => s.to_string(),
            OtelString::RefCounted(s) => s.to_string(),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            OtelString::Owned(s) => s.fmt(fmt),
            OtelString::Static(s) => s.fmt(fmt),
            OtelString::RefCounted(s) => s.fmt(fmt),
        }
    }
}

#[derive(Clone, Debug, Eq)]
enum OtelString {
    Owned(Box<str>),
    Static(&'static str),
    RefCounted(Arc<str>),
}

impl OtelString {
    fn as_str(&self) -> &str {
        match self {
            OtelString::Owned(s) => s.as_ref(),
            OtelString::Static(s) => s,
            OtelString::RefCounted(s) => s.as_ref(),
        }
    }
}

impl PartialOrd for OtelString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OtelString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq for OtelString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl hash::Hash for OtelString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

/// A [Value::Array] containing homogeneous values.
#[derive(Clone, Debug, PartialEq)]
pub enum Array {
    /// Array of bools
    Bool(Vec<bool>),
    /// Array of integers
    I64(Vec<i64>),
    /// Array of floats
    F64(Vec<f64>),
    /// Array of strings
    String(Vec<StringValue>),
}

impl fmt::Display for Array {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Array::Bool(values) => display_array_str(values, fmt),
            Array::I64(values) => display_array_str(values, fmt),
            Array::F64(values) => display_array_str(values, fmt),
            Array::String(values) => {
                write!(fmt, "[")?;
                for (i, t) in values.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, ",")?;
                    }
                    write!(fmt, "\"{}\"", t)?;
                }
                write!(fmt, "]")
            }
        }
    }
}

fn display_array_str<T: fmt::Display>(slice: &[T], fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(fmt, "[")?;
    for (i, t) in slice.iter().enumerate() {
        if i > 0 {
            write!(fmt, ",")?;
        }
        write!(fmt, "{}", t)?;
    }
    write!(fmt, "]")
}

macro_rules! into_array {
    ($(($t:ty, $val:expr),)+) => {
        $(
            impl From<$t> for Array {
                fn from(t: $t) -> Self {
                    $val(t)
                }
            }
        )+
    }
}

into_array!(
    (Vec<bool>, Array::Bool),
    (Vec<i64>, Array::I64),
    (Vec<f64>, Array::F64),
    (Vec<StringValue>, Array::String),
);

/// The value part of attribute [KeyValue] pairs.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// bool values
    Bool(bool),
    /// i64 values
    I64(i64),
    /// f64 values
    F64(f64),
    /// String values
    String(StringValue),
    /// Array of homogeneous values
    Array(Array),
}

/// Wrapper for string-like values
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringValue(OtelString);

impl fmt::Debug for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            OtelString::Owned(s) => s.fmt(f),
            OtelString::Static(s) => s.fmt(f),
            OtelString::RefCounted(s) => s.fmt(f),
        }
    }
}

impl AsRef<str> for StringValue {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl StringValue {
    /// Returns a string slice to this value
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<StringValue> for String {
    fn from(s: StringValue) -> Self {
        match s.0 {
            OtelString::Owned(s) => s.to_string(),
            OtelString::Static(s) => s.to_string(),
            OtelString::RefCounted(s) => s.to_string(),
        }
    }
}

impl From<&'static str> for StringValue {
    fn from(s: &'static str) -> Self {
        StringValue(OtelString::Static(s))
    }
}

impl From<String> for StringValue {
    fn from(s: String) -> Self {
        StringValue(OtelString::Owned(s.into_boxed_str()))
    }
}

impl From<Arc<str>> for StringValue {
    fn from(s: Arc<str>) -> Self {
        StringValue(OtelString::RefCounted(s))
    }
}

impl From<Cow<'static, str>> for StringValue {
    fn from(s: Cow<'static, str>) -> Self {
        match s {
            Cow::Owned(s) => StringValue(OtelString::Owned(s.into_boxed_str())),
            Cow::Borrowed(s) => StringValue(OtelString::Static(s)),
        }
    }
}

impl Value {
    /// String representation of the `Value`
    ///
    /// This will allocate iff the underlying value is not a `String`.
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Value::Bool(v) => format!("{}", v).into(),
            Value::I64(v) => format!("{}", v).into(),
            Value::F64(v) => format!("{}", v).into(),
            Value::String(v) => Cow::Borrowed(v.as_str()),
            Value::Array(v) => format!("{}", v).into(),
        }
    }
}

macro_rules! from_values {
   (
        $(
            ($t:ty, $val:expr);
        )+
    ) => {
        $(
            impl From<$t> for Value {
                fn from(t: $t) -> Self {
                    $val(t)
                }
            }
        )+
    }
}

from_values!(
    (bool, Value::Bool);
    (i64, Value::I64);
    (f64, Value::F64);
    (StringValue, Value::String);
);

impl From<&'static str> for Value {
    fn from(s: &'static str) -> Self {
        Value::String(s.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s.into())
    }
}

impl From<Arc<str>> for Value {
    fn from(s: Arc<str>) -> Self {
        Value::String(s.into())
    }
}

impl From<Cow<'static, str>> for Value {
    fn from(s: Cow<'static, str>) -> Self {
        Value::String(s.into())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => v.fmt(fmt),
            Value::I64(v) => v.fmt(fmt),
            Value::F64(v) => v.fmt(fmt),
            Value::String(v) => fmt.write_str(v.as_str()),
            Value::Array(v) => v.fmt(fmt),
        }
    }
}

/// A key-value pair describing an attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue {
    /// The attribute name
    pub key: Key,

    /// The attribute value
    pub value: Value,
}

impl KeyValue {
    /// Create a new `KeyValue` pair.
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        KeyValue {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct F64Hashable(f64);

impl PartialEq for F64Hashable {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F64Hashable {}

impl Hash for F64Hashable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Hash for KeyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        match &self.value {
            Value::F64(f) => F64Hashable(*f).hash(state),
            Value::Array(a) => match a {
                Array::Bool(b) => b.hash(state),
                Array::I64(i) => i.hash(state),
                Array::F64(f) => f.iter().for_each(|f| F64Hashable(*f).hash(state)),
                Array::String(s) => s.hash(state),
            },
            Value::Bool(b) => b.hash(state),
            Value::I64(i) => i.hash(state),
            Value::String(s) => s.hash(state),
        };
    }
}

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl Eq for KeyValue {}

/// Marker trait for errors returned by exporters
pub trait ExportError: std::error::Error + Send + Sync + 'static {
    /// The name of exporter that returned this error
    fn exporter_name(&self) -> &'static str;
}

/// Information about a library or crate providing instrumentation.
///
/// An instrumentation library should be named to follow any naming conventions
/// of the instrumented library (e.g. 'middleware' for a web framework).
///
/// See the [instrumentation libraries] spec for more information.
///
/// [instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct InstrumentationLibrary {
    /// The library name.
    ///
    /// This should be the name of the crate providing the instrumentation.
    pub name: Cow<'static, str>,

    /// The library version.
    ///
    /// # Examples
    ///
    /// ```
    /// let library = opentelemetry::InstrumentationLibrary::builder("my-crate").
    ///     with_version(env!("CARGO_PKG_VERSION")).
    ///     with_schema_url("https://opentelemetry.io/schemas/1.17.0").
    ///     build();
    /// ```
    pub version: Option<Cow<'static, str>>,

    /// [Schema url] used by this library.
    ///
    /// [Schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    pub schema_url: Option<Cow<'static, str>>,

    /// Specifies the instrumentation scope attributes to associate with emitted telemetry.
    pub attributes: Vec<KeyValue>,
}

// Uniqueness for InstrumentationLibrary/InstrumentationScope does not depend on attributes
impl Eq for InstrumentationLibrary {}

impl PartialEq for InstrumentationLibrary {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.version == other.version
            && self.schema_url == other.schema_url
    }
}

impl hash::Hash for InstrumentationLibrary {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
        self.schema_url.hash(state);
    }
}

impl InstrumentationLibrary {
    /// Deprecated, use [`InstrumentationLibrary::builder()`]
    ///
    /// Create an new instrumentation library.
    #[deprecated(since = "0.23.0", note = "Please use builder() instead")]
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> InstrumentationLibrary {
        InstrumentationLibrary {
            name: name.into(),
            version: version.map(Into::into),
            schema_url: schema_url.map(Into::into),
            attributes: attributes.unwrap_or_default(),
        }
    }

    /// Create a new builder to create an [InstrumentationLibrary]
    pub fn builder<T: Into<Cow<'static, str>>>(name: T) -> InstrumentationLibraryBuilder {
        InstrumentationLibraryBuilder {
            name: name.into(),
            version: None,
            schema_url: None,
            attributes: None,
        }
    }
}

/// Configuration options for [InstrumentationLibrary].
///
/// An instrumentation library is a library or crate providing instrumentation.
/// It should be named to follow any naming conventions of the instrumented
/// library (e.g. 'middleware' for a web framework).
///
/// Apart from the name, all other fields are optional.
///
/// See the [instrumentation libraries] spec for more information.
///
/// [instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries
#[derive(Debug)]
pub struct InstrumentationLibraryBuilder {
    name: Cow<'static, str>,

    version: Option<Cow<'static, str>>,

    schema_url: Option<Cow<'static, str>>,

    attributes: Option<Vec<KeyValue>>,
}

impl InstrumentationLibraryBuilder {
    /// Configure the version for the instrumentation library
    ///
    /// # Examples
    ///
    /// ```
    /// let library = opentelemetry::InstrumentationLibrary::builder("my-crate")
    ///     .with_version("v0.1.0")
    ///     .build();
    /// ```
    pub fn with_version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Configure the Schema URL for the instrumentation library
    ///
    /// # Examples
    ///
    /// ```
    /// let library = opentelemetry::InstrumentationLibrary::builder("my-crate")
    ///     .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
    ///     .build();
    /// ```
    pub fn with_schema_url(mut self, schema_url: impl Into<Cow<'static, str>>) -> Self {
        self.schema_url = Some(schema_url.into());
        self
    }

    /// Configure the attributes for the instrumentation library
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::KeyValue;
    ///
    /// let library = opentelemetry::InstrumentationLibrary::builder("my-crate")
    ///     .with_attributes(vec![KeyValue::new("k", "v")])
    ///     .build();
    /// ```
    pub fn with_attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = KeyValue>,
    {
        self.attributes = Some(attributes.into_iter().collect());
        self
    }

    /// Create a new [InstrumentationLibrary] from this configuration
    pub fn build(self) -> InstrumentationLibrary {
        InstrumentationLibrary {
            name: self.name,
            version: self.version,
            schema_url: self.schema_url,
            attributes: self.attributes.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::KeyValue;
    use std::hash::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn equality_kv_float() {
        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.0);
        assert_eq!(kv1, kv2);

        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.01);
        assert_ne!(kv1, kv2);

        let kv1 = KeyValue::new("key", std::f64::NAN);
        let kv2 = KeyValue::new("key", std::f64::NAN);
        assert_ne!(kv1, kv2, "NAN is not equal to itself");

        let kv1 = KeyValue::new("key", std::f64::INFINITY);
        let kv2 = KeyValue::new("key", std::f64::INFINITY);
        assert_eq!(kv1, kv2);

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(kv1, kv2);
        }
    }

    #[test]
    fn hash_kv_float() {
        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.0);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", 1.001);
        let kv2 = KeyValue::new("key", 1.001);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", std::f64::NAN);
        let kv2 = KeyValue::new("key", std::f64::NAN);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", std::f64::INFINITY);
        let kv2 = KeyValue::new("key", std::f64::INFINITY);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(hash_helper(&kv1), hash_helper(&kv2));
        }
    }

    fn hash_helper<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}
