use crate::{trace::Tracer, InstrumentationLibrary, InstrumentationLibraryBuilder, KeyValue};
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
    /// use opentelemetry::{global, trace::TracerProvider};
    /// use opentelemetry::KeyValue;
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer("my_app");
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.tracer_builder("my_library").
    ///     with_version(env!("CARGO_PKG_VERSION")).
    ///     with_schema_url("https://opentelemetry.io/schema/1.0.0").
    ///     with_attributes(vec![KeyValue::new("key", "value")]).
    ///     build();
    /// ```
    fn tracer(&self, name: impl Into<Cow<'static, str>>) -> Self::Tracer {
        self.tracer_builder(name).build()
    }

    /// Deprecated, use [`TracerProvider::tracer_builder()`]
    ///
    /// Returns a new versioned tracer with a given name.
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, trace::TracerProvider};
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
    #[deprecated(since = "0.23.0", note = "Please use tracer_builder() instead")]
    fn versioned_tracer(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<impl Into<Cow<'static, str>>>,
        schema_url: Option<impl Into<Cow<'static, str>>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Tracer {
        let mut builder = self.tracer_builder(name);
        if let Some(v) = version {
            builder = builder.with_version(v);
        }
        if let Some(s) = schema_url {
            builder = builder.with_version(s);
        }
        if let Some(a) = attributes {
            builder = builder.with_attributes(a);
        }

        builder.build()
    }

    /// Returns a new builder for creating a [`Tracer`] instance
    ///
    /// The `name` should be the application name or the name of the library
    /// providing instrumentation. If the name is empty, then an
    /// implementation-defined default name may be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, trace::TracerProvider};
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer_builder("my_app").build();
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let tracer = provider.tracer_builder("my_library")
    ///     .with_version(env!("CARGO_PKG_VERSION"))
    ///     .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    ///     .build();
    /// ```
    fn tracer_builder(&self, name: impl Into<Cow<'static, str>>) -> TracerBuilder<'_, Self> {
        TracerBuilder {
            provider: self,
            library_builder: InstrumentationLibrary::builder(name),
        }
    }

    /// Returns a new versioned tracer with the given instrumentation library.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{global, InstrumentationLibrary, trace::TracerProvider};
    ///
    /// let provider = global::tracer_provider();
    ///
    /// // tracer used in applications/binaries
    /// let tracer = provider.tracer("my_app");
    ///
    /// // tracer used in libraries/crates that optionally includes version and schema url
    /// let library = std::sync::Arc::new(
    ///     InstrumentationLibrary::builder(env!("CARGO_PKG_NAME"))
    ///         .with_version(env!("CARGO_PKG_VERSION"))
    ///         .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    ///         .build(),
    /// );
    ///
    /// let tracer = provider.library_tracer(library);
    /// ```
    fn library_tracer(&self, library: Arc<InstrumentationLibrary>) -> Self::Tracer;
}

#[derive(Debug)]
pub struct TracerBuilder<'a, T: TracerProvider + ?Sized> {
    provider: &'a T,
    library_builder: InstrumentationLibraryBuilder,
}

impl<'a, T: TracerProvider + ?Sized> TracerBuilder<'a, T> {
    pub fn with_version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.library_builder = self.library_builder.with_version(version);
        self
    }

    pub fn with_schema_url(mut self, schema_url: impl Into<Cow<'static, str>>) -> Self {
        self.library_builder = self.library_builder.with_schema_url(schema_url);
        self
    }

    pub fn with_attributes(mut self, attributes: impl Into<Vec<KeyValue>>) -> Self {
        self.library_builder = self.library_builder.with_attributes(attributes);
        self
    }

    pub fn build(self) -> T::Tracer {
        self.provider
            .library_tracer(Arc::new(self.library_builder.build()))
    }
}
