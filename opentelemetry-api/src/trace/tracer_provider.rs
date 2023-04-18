use crate::{trace::Tracer, KeyValue};
use std::borrow::Cow;

/// Types that can create instances of [`Tracer`].
///
/// See the [`global`] module for examples of storing and retrieving tracer
/// provider instances.
///
/// [`global`]: crate::global
pub trait TracerProvider {
    /// The [`Tracer`] type that this provider will return.
    type Tracer: Tracer;

    /// Returns a new tracer with the given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::{global, trace::TracerProvider};
    /// use opentelemetry_api::KeyValue;
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer("my_app".into());
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.versioned_tracer(
    ///     "my_library".into(),
    ///     Some(env!("CARGO_PKG_VERSION").into()),
    ///     Some("https://opentelemetry.io/schema/1.0.0".into()),
    ///     Some(vec![KeyValue::new("key", "value")]),
    /// );
    /// ```
    fn tracer(&self, name: Cow<'static, str>) -> Self::Tracer {
        self.versioned_tracer(name, None, None, None)
    }

    /// Returns a new versioned tracer with a given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::{global, trace::TracerProvider};
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer("my_app".into());
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.versioned_tracer(
    ///     "my_library".into(),
    ///     Some(env!("CARGO_PKG_VERSION").into()),
    ///     Some("https://opentelemetry.io/schema/1.0.0".into()),
    ///     None,
    /// );
    /// ```
    fn versioned_tracer(
        &self,
        name: Cow<'static, str>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Tracer;
}
