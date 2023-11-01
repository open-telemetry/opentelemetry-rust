#[cfg(feature = "trace")]
use crate::trace::context::SynchronizedSpan;
use futures_core::stream::Stream;
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::thread;

thread_local! {
    static CURRENT_CONTEXT: RefCell<Context> = RefCell::new(Context::default());
}

/// An execution-scoped collection of values.
///
/// A [`Context`] is a propagation mechanism which carries execution-scoped
/// values across API boundaries and between logically associated execution
/// units. Cross-cutting concerns access their data in-process using the same
/// shared context object.
///
/// [`Context`]s are immutable, and their write operations result in the creation
/// of a new context containing the original values and the new specified values.
///
/// ## Context state
///
/// Concerns can create and retrieve their local state in the current execution
/// state represented by a context through the [`get`] and [`with_value`]
/// methods. It is recommended to use application-specific types when storing new
/// context values to avoid unintentionally overwriting existing state.
///
/// ## Managing the current context
///
/// Contexts can be associated with the caller's current execution unit on a
/// given thread via the [`attach`] method, and previous contexts can be restored
/// by dropping the returned [`ContextGuard`]. Context can be nested, and will
/// restore their parent outer context when detached on drop. To access the
/// values of the context, a snapshot can be created via the [`Context::current`]
/// method.
///
/// [`Context::current`]: Context::current()
/// [`get`]: Context::get()
/// [`with_value`]: Context::with_value()
/// [`attach`]: Context::attach()
///
/// # Examples
///
/// ```
/// use opentelemetry::Context;
///
/// // Application-specific `a` and `b` values
/// #[derive(Debug, PartialEq)]
/// struct ValueA(&'static str);
/// #[derive(Debug, PartialEq)]
/// struct ValueB(u64);
///
/// let _outer_guard = Context::new().with_value(ValueA("a")).attach();
///
/// // Only value a has been set
/// let current = Context::current();
/// assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
/// assert_eq!(current.get::<ValueB>(), None);
///
/// {
///     let _inner_guard = Context::current_with_value(ValueB(42)).attach();
///     // Both values are set in inner context
///     let current = Context::current();
///     assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
///     assert_eq!(current.get::<ValueB>(), Some(&ValueB(42)));
/// }
///
/// // Resets to only the `a` value when inner guard is dropped
/// let current = Context::current();
/// assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
/// assert_eq!(current.get::<ValueB>(), None);
/// ```
#[derive(Clone, Default)]
pub struct Context {
    #[cfg(feature = "trace")]
    pub(super) span: Option<Arc<SynchronizedSpan>>,
    pub suppress_logs: bool,
    entries: HashMap<TypeId, Arc<dyn Any + Sync + Send>, BuildHasherDefault<IdHasher>>,
}

impl Context {
    /// Creates an empty `Context`.
    ///
    /// The context is initially created with a capacity of 0, so it will not
    /// allocate. Use [`with_value`] to create a new context that has entries.
    ///
    /// [`with_value`]: Context::with_value()
    pub fn new() -> Self {
        Context::default()
    }

    /// Returns an immutable snapshot of the current thread's context.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    ///
    /// fn do_work() {
    ///     assert_eq!(Context::current().get(), Some(&ValueA("a")));
    /// }
    ///
    /// let _guard = Context::new().with_value(ValueA("a")).attach();
    /// do_work()
    /// ```
    pub fn current() -> Self {
        Context::map_current(|cx| cx.clone())
    }

    /// Applies a function to the current context returning its value.
    ///
    /// This can be used to build higher performing algebraic expressions for
    /// optionally creating a new context without the overhead of cloning the
    /// current one and dropping it.
    ///
    /// Note: This function will panic if you attempt to attach another context
    /// while the current one is still borrowed.
    pub fn map_current<T>(f: impl FnOnce(&Context) -> T) -> T {
        CURRENT_CONTEXT.with(|cx| f(&cx.borrow()))
    }

    /// Returns a clone of the current thread's context with the given value.
    ///
    /// This is a more efficient form of `Context::current().with_value(value)`
    /// as it avoids the intermediate context clone.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// // Given some value types defined in your application
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    /// #[derive(Debug, PartialEq)]
    /// struct ValueB(u64);
    ///
    /// // You can create and attach context with the first value set to "a"
    /// let _guard = Context::new().with_value(ValueA("a")).attach();
    ///
    /// // And create another context based on the fist with a new value
    /// let all_current_and_b = Context::current_with_value(ValueB(42));
    ///
    /// // The second context now contains all the current values and the addition
    /// assert_eq!(all_current_and_b.get::<ValueA>(), Some(&ValueA("a")));
    /// assert_eq!(all_current_and_b.get::<ValueB>(), Some(&ValueB(42)));
    /// ```
    pub fn current_with_value<T: 'static + Send + Sync>(value: T) -> Self {
        let mut new_context = Context::current();
        new_context
            .entries
            .insert(TypeId::of::<T>(), Arc::new(value));

        new_context
    }

    /// Returns a reference to the entry for the corresponding value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// // Given some value types defined in your application
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    /// #[derive(Debug, PartialEq)]
    /// struct MyUser();
    ///
    /// let cx = Context::new().with_value(ValueA("a"));
    ///
    /// // Values can be queried by type
    /// assert_eq!(cx.get::<ValueA>(), Some(&ValueA("a")));
    ///
    /// // And return none if not yet set
    /// assert_eq!(cx.get::<MyUser>(), None);
    /// ```
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.entries
            .get(&TypeId::of::<T>())
            .and_then(|rc| rc.downcast_ref())
    }

    /// Returns a copy of the context with the new value included.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// // Given some value types defined in your application
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    /// #[derive(Debug, PartialEq)]
    /// struct ValueB(u64);
    ///
    /// // You can create a context with the first value set to "a"
    /// let cx_with_a = Context::new().with_value(ValueA("a"));
    ///
    /// // And create another context based on the fist with a new value
    /// let cx_with_a_and_b = cx_with_a.with_value(ValueB(42));
    ///
    /// // The first context is still available and unmodified
    /// assert_eq!(cx_with_a.get::<ValueA>(), Some(&ValueA("a")));
    /// assert_eq!(cx_with_a.get::<ValueB>(), None);
    ///
    /// // The second context now contains both values
    /// assert_eq!(cx_with_a_and_b.get::<ValueA>(), Some(&ValueA("a")));
    /// assert_eq!(cx_with_a_and_b.get::<ValueB>(), Some(&ValueB(42)));
    /// ```
    pub fn with_value<T: 'static + Send + Sync>(&self, value: T) -> Self {
        let mut new_context = self.clone();
        new_context
            .entries
            .insert(TypeId::of::<T>(), Arc::new(value));

        new_context
    }

    /// Replaces the current context on this thread with this context.
    ///
    /// Dropping the returned [`ContextGuard`] will reset the current context to the
    /// previous value.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    ///
    /// let my_cx = Context::new().with_value(ValueA("a"));
    ///
    /// // Set the current thread context
    /// let cx_guard = my_cx.attach();
    /// assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA("a")));
    ///
    /// // Drop the guard to restore the previous context
    /// drop(cx_guard);
    /// assert_eq!(Context::current().get::<ValueA>(), None);
    /// ```
    ///
    /// Guards do not need to be explicitly dropped:
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    ///
    /// fn my_function() -> String {
    ///     // attach a context the duration of this function.
    ///     let my_cx = Context::new().with_value(ValueA("a"));
    ///     // NOTE: a variable name after the underscore is **required** or rust
    ///     // will drop the guard, restoring the previous context _immediately_.
    ///     let _guard = my_cx.attach();
    ///
    ///     // anything happening in functions we call can still access my_cx...
    ///     my_other_function();
    ///
    ///     // returning from the function drops the guard, exiting the span.
    ///     return "Hello world".to_owned();
    /// }
    ///
    /// fn my_other_function() {
    ///     // ...
    /// }
    /// ```
    /// Sub-scopes may be created to limit the duration for which the span is
    /// entered:
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct ValueA(&'static str);
    ///
    /// let my_cx = Context::new().with_value(ValueA("a"));
    ///
    /// {
    ///     let _guard = my_cx.attach();
    ///
    ///     // the current context can access variables in
    ///     assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA("a")));
    ///
    ///     // exiting the scope drops the guard, detaching the context.
    /// }
    ///
    /// // this is back in the default empty context
    /// assert_eq!(Context::current().get::<ValueA>(), None);
    /// ```
    pub fn attach(self) -> ContextGuard {
        let previous_cx = CURRENT_CONTEXT
            .try_with(|current| current.replace(self))
            .ok();

        ContextGuard {
            previous_cx,
            _marker: PhantomData,
        }
    }

    #[cfg(feature = "trace")]
    pub(super) fn current_with_synchronized_span(value: SynchronizedSpan) -> Self {
        Context {
            span: Some(Arc::new(value)),
            entries: Context::map_current(|cx| cx.entries.clone()),
            ..Context::default()
        }
    }

    #[cfg(feature = "trace")]
    pub(super) fn with_synchronized_span(&self, value: SynchronizedSpan) -> Self {
        Context {
            span: Some(Arc::new(value)),
            entries: self.entries.clone(),
            ..self.clone()
        }
    }

    pub(super) fn with_suppression(&self) -> Self {
        Context {
            suppress_logs: true,
            entries: self.entries.clone(),
            ..self.clone()
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("entries", &self.entries.len())
            .finish()
    }
}

pin_project! {
    /// A future, stream, or sink that has an associated context.
    #[derive(Clone, Debug)]
    pub struct WithContext<T> {
        #[pin]
        inner: T,
        otel_cx: Context,
    }
}

impl<T: Sized> FutureExt for T {}

impl<T: std::future::Future> std::future::Future for WithContext<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();

        this.inner.poll(task_cx)
    }
}

impl<T: Stream> Stream for WithContext<T> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_next(this.inner, task_cx)
    }
}

impl<I, T: Sink<I>> Sink<I> for WithContext<T>
where
    T: Sink<I>,
{
    type Error = T::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_ready(this.inner, task_cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::start_send(this.inner, item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_flush(this.inner, task_cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _enter = this.otel_cx.clone().attach();
        T::poll_close(this.inner, task_cx)
    }
}

/// Extension trait allowing futures, streams, and sinks to be traced with a span.
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as current while it is being polled.
    ///
    /// [`Context`]: crate::Context
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as the default while it is being polled.
    ///
    /// [`Context`]: crate::Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }

    /// Attaches the current context [`Context`] to this type, with
    /// suppression enabled, returning a `WithContext` wrapper.
    fn with_current_context_supp(self) -> WithContext<Self> {
        let otel_cx = Context::current().with_suppression();
        self.with_context(otel_cx)
    }
}

/// A guard that resets the current context to the prior context when dropped.
#[allow(missing_debug_implementations)]
pub struct ContextGuard {
    previous_cx: Option<Context>,
    // ensure this type is !Send as it relies on thread locals
    _marker: PhantomData<*const ()>,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        if let Some(previous_cx) = self.previous_cx.take() {
            let _ = CURRENT_CONTEXT.try_with(|current| current.replace(previous_cx));
        }
    }
}

/// With TypeIds as keys, there's no need to hash them. They are already hashes
/// themselves, coming from the compiler. The IdHasher holds the u64 of
/// the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Clone, Default, Debug)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_contexts() {
        #[derive(Debug, PartialEq)]
        struct ValueA(&'static str);
        #[derive(Debug, PartialEq)]
        struct ValueB(u64);
        let _outer_guard = Context::new().with_value(ValueA("a")).attach();

        // Only value `a` is set
        let current = Context::current();
        assert_eq!(current.get(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        {
            let _inner_guard = Context::current_with_value(ValueB(42)).attach();
            // Both values are set in inner context
            let current = Context::current();
            assert_eq!(current.get(), Some(&ValueA("a")));
            assert_eq!(current.get(), Some(&ValueB(42)));

            assert!(Context::map_current(|cx| {
                assert_eq!(cx.get(), Some(&ValueA("a")));
                assert_eq!(cx.get(), Some(&ValueB(42)));
                true
            }));
        }

        // Resets to only value `a` when inner guard is dropped
        let current = Context::current();
        assert_eq!(current.get(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        assert!(Context::map_current(|cx| {
            assert_eq!(cx.get(), Some(&ValueA("a")));
            assert_eq!(cx.get::<ValueB>(), None);
            true
        }));
    }
}
