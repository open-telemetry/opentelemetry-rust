use crate::{trace::Tracer, InstrumentationLibrary};
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
    /// ```
    fn tracer(&self, name: impl Into<Cow<'static, str>>) -> Self::Tracer {
        let library = InstrumentationLibrary::builder(name).build();
        self.library_tracer(Arc::new(library))
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
