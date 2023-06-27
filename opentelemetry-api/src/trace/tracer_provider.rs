use crate::{trace::Tracer, InstrumentationLibrary, KeyValue};
use std::{borrow::Cow, sync::Arc};

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
    /// let tracer = provider.tracer("my_app");
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.versioned_tracer(
    ///     "my_library",
    ///     Some(env!("CARGO_PKG_VERSION")),
    ///     Some("https://opentelemetry.io/schema/1.0.0"),
    ///     Some(vec![KeyValue::new("key", "value")]),
    /// );
    /// ```
    fn tracer(&self, name: impl Into<Cow<'static, str>>) -> Self::Tracer {
        self.versioned_tracer(
            name,
            None::<Cow<'static, str>>,
            None::<Cow<'static, str>>,
            None,
        )
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
    /// let tracer = provider.tracer("my_app");
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.versioned_tracer(
    ///     "my_library",
    ///     Some(env!("CARGO_PKG_VERSION")),
    ///     Some("https://opentelemetry.io/schema/1.0.0"),
    ///     None,
    /// );
    /// ```
    fn versioned_tracer(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Tracer {
        self.library_tracer(Arc::new(InstrumentationLibrary::new(
            name, version, schema_url, attributes,
        )))
    }

    /// Returns a new versioned tracer with the given instrumentation library.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_api::{global, InstrumentationLibrary, trace::TracerProvider};
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer("my_app");
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let library = std::sync::Arc::new(InstrumentationLibrary::new(
    ///     env!("CARGO_PKG_NAME"),
    ///     Some(env!("CARGO_PKG_VERSION")),
    ///     Some("https://opentelemetry.io/schema/1.0.0"),
    ///     None,
    /// ));
    /// let tracer = provider.library_tracer(library);
    /// ```
    fn library_tracer(&self, library: Arc<InstrumentationLibrary>) -> Self::Tracer;
}
