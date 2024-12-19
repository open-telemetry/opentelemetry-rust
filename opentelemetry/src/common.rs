use std::borrow::{Borrow, Cow};
use std::sync::Arc;
use std::{fmt, hash};

/// The key part of attribute [KeyValue] pairs.
///
/// See the [attribute naming] spec for guidelines.
///
/// [attribute naming]: https://github.com/open-telemetry/semantic-conventions/blob/main/docs/general/attribute-naming.md
#[non_exhaustive]
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

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.0.as_str()
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Hash)]
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
#[non_exhaustive]
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

/// Information about a library or crate providing instrumentation.
///
/// An instrumentation scope should be named to follow any naming conventions
/// of the instrumented library (e.g. 'middleware' for a web framework).
///
/// See the [instrumentation libraries] spec for more information.
///
/// [instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct InstrumentationScope {
    /// The library name.
    ///
    /// This should be the name of the crate providing the instrumentation.
    name: Cow<'static, str>,

    /// The library version.
    version: Option<Cow<'static, str>>,

    /// [Schema URL] used by this library.
    ///
    /// [Schema URL]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    schema_url: Option<Cow<'static, str>>,

    /// Specifies the instrumentation scope attributes to associate with emitted telemetry.
    attributes: Vec<KeyValue>,
}

// Uniqueness for InstrumentationScope does not depend on attributes
impl Eq for InstrumentationScope {}

impl PartialEq for InstrumentationScope {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.version == other.version
            && self.schema_url == other.schema_url
    }
}

impl hash::Hash for InstrumentationScope {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
        self.schema_url.hash(state);
    }
}

impl InstrumentationScope {
    /// Create a new builder to create an [InstrumentationScope]
    pub fn builder<T: Into<Cow<'static, str>>>(name: T) -> InstrumentationScopeBuilder {
        InstrumentationScopeBuilder {
            name: name.into(),
            version: None,
            schema_url: None,
            attributes: None,
        }
    }

    /// Returns the instrumentation library name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the instrumentation library version.
    #[inline]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns the [Schema URL] used by this library.
    ///
    /// [Schema URL]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
    #[inline]
    pub fn schema_url(&self) -> Option<&str> {
        self.schema_url.as_deref()
    }

    /// Returns the instrumentation scope attributes to associate with emitted telemetry.
    #[inline]
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }
}

/// Configuration options for [InstrumentationScope].
///
/// An instrumentation scope is a library or crate providing instrumentation.
/// It should be named to follow any naming conventions of the instrumented
/// library (e.g. 'middleware' for a web framework).
///
/// Apart from the name, all other fields are optional.
///
/// See the [instrumentation libraries] spec for more information.
///
/// [instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries
#[derive(Debug)]
pub struct InstrumentationScopeBuilder {
    name: Cow<'static, str>,
    version: Option<Cow<'static, str>>,
    schema_url: Option<Cow<'static, str>>,
    attributes: Option<Vec<KeyValue>>,
}

impl InstrumentationScopeBuilder {
    /// Configure the version for the instrumentation scope
    ///
    /// # Examples
    ///
    /// ```
    /// let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    ///     .with_version("v0.1.0")
    ///     .build();
    /// ```
    pub fn with_version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Configure the Schema URL for the instrumentation scope
    ///
    /// # Examples
    ///
    /// ```
    /// let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    ///     .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
    ///     .build();
    /// ```
    pub fn with_schema_url(mut self, schema_url: impl Into<Cow<'static, str>>) -> Self {
        self.schema_url = Some(schema_url.into());
        self
    }

    /// Configure the attributes for the instrumentation scope
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::KeyValue;
    ///
    /// let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    ///     .with_attributes([KeyValue::new("k", "v")])
    ///     .build();
    /// ```
    pub fn with_attributes<I>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = KeyValue>,
    {
        self.attributes = Some(attributes.into_iter().collect());
        self
    }

    /// Create a new [InstrumentationScope] from this configuration
    pub fn build(self) -> InstrumentationScope {
        InstrumentationScope {
            name: self.name,
            version: self.version,
            schema_url: self.schema_url,
            attributes: self.attributes.unwrap_or_default(),
        }
    }
}
