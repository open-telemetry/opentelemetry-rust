//! Execution-scoped context propagation.
//!
//! The `context` module provides mechanisms for propagating values across API boundaries and between
//! logically associated execution units. It enables cross-cutting concerns to access their data in-process
//! using a shared context object.
//!
//! # Main Types
//!
//! - [`Context`]: An immutable, execution-scoped collection of values.
//!

use crate::otel_warn;
#[cfg(feature = "trace")]
use crate::trace::context::SynchronizedSpan;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

#[cfg(test)]
mod tests;

#[cfg(feature = "futures")]
mod future_ext;

#[cfg(feature = "futures")]
#[allow(deprecated)]
pub use future_ext::FutureExt;

#[cfg(feature = "futures")]
pub use future_ext::{FutureContextExt, SinkContextExt, StreamContextExt, WithContext};

thread_local! {
    static CURRENT_CONTEXT: RefCell<ContextStack> = RefCell::new(ContextStack::default());
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
    pub(crate) span: Option<Arc<SynchronizedSpan>>,
    entries: Option<Arc<EntryMap>>,
    suppress_telemetry: bool,
}

type EntryMap = HashMap<TypeId, Arc<dyn Any + Sync + Send>, BuildHasherDefault<IdHasher>>;

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
        Self::map_current(|cx| cx.clone())
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
        CURRENT_CONTEXT.with(|cx| cx.borrow().map_current_cx(f))
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
        Self::map_current(|cx| cx.with_value(value))
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
            .as_ref()?
            .get(&TypeId::of::<T>())?
            .downcast_ref()
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
        let entries = if let Some(current_entries) = &self.entries {
            let mut inner_entries = (**current_entries).clone();
            inner_entries.insert(TypeId::of::<T>(), Arc::new(value));
            Some(Arc::new(inner_entries))
        } else {
            let mut entries = EntryMap::default();
            entries.insert(TypeId::of::<T>(), Arc::new(value));
            Some(Arc::new(entries))
        };
        Context {
            entries,
            #[cfg(feature = "trace")]
            span: self.span.clone(),
            suppress_telemetry: self.suppress_telemetry,
        }
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
        let cx_id = CURRENT_CONTEXT.with(|cx| cx.borrow_mut().push(self));

        ContextGuard {
            cx_pos: cx_id,
            _marker: PhantomData,
        }
    }

    /// Returns whether telemetry is suppressed in this context.
    #[inline]
    pub fn is_telemetry_suppressed(&self) -> bool {
        self.suppress_telemetry
    }

    /// Returns a new context with telemetry suppression enabled.
    pub fn with_telemetry_suppressed(&self) -> Self {
        Context {
            entries: self.entries.clone(),
            #[cfg(feature = "trace")]
            span: self.span.clone(),
            suppress_telemetry: true,
        }
    }

    /// Enters a scope where telemetry is suppressed.
    ///
    /// This method is specifically designed for OpenTelemetry components (like Exporters,
    /// Processors etc.) to prevent generating recursive or self-referential
    /// telemetry data when performing their own operations.
    ///
    /// Without suppression, we have a telemetry-induced-telemetry situation
    /// where, operations like exporting telemetry could generate new telemetry
    /// about the export process itself, potentially causing:
    /// - Infinite telemetry feedback loops
    /// - Excessive resource consumption
    ///
    /// This method:
    /// 1. Takes the current context
    /// 2. Creates a new context from current, with `suppress_telemetry` set to `true`
    /// 3. Attaches it to the current thread
    /// 4. Returns a guard that restores the previous context when dropped
    ///
    /// OTel SDK components would check `is_current_telemetry_suppressed()` before
    /// generating new telemetry, but not end users.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::Context;
    ///
    /// // Example: Inside an exporter's implementation
    /// fn example_export_function() {
    ///     // Prevent telemetry-generating operations from creating more telemetry
    ///     let _guard = Context::enter_telemetry_suppressed_scope();
    ///     
    ///     // Verify suppression is active
    ///     assert_eq!(Context::is_current_telemetry_suppressed(), true);
    ///     
    ///     // Here you would normally perform operations that might generate telemetry
    ///     // but now they won't because the context has suppression enabled
    /// }
    ///
    /// // Demonstrate the function
    /// example_export_function();
    /// ```
    pub fn enter_telemetry_suppressed_scope() -> ContextGuard {
        Self::map_current(|cx| cx.with_telemetry_suppressed()).attach()
    }

    /// Returns whether telemetry is suppressed in the current context.
    ///
    /// This method is used by OpenTelemetry components to determine whether they should
    /// generate new telemetry in the current execution context. It provides a performant
    /// way to check the suppression state.
    ///
    /// End-users generally should not use this method directly, as it is primarily intended for
    /// OpenTelemetry SDK components.
    ///
    ///
    #[inline]
    pub fn is_current_telemetry_suppressed() -> bool {
        Self::map_current(|cx| cx.is_telemetry_suppressed())
    }

    #[cfg(feature = "trace")]
    pub(crate) fn current_with_synchronized_span(value: SynchronizedSpan) -> Self {
        Self::map_current(|cx| Context {
            span: Some(Arc::new(value)),
            entries: cx.entries.clone(),
            suppress_telemetry: cx.suppress_telemetry,
        })
    }

    #[cfg(feature = "trace")]
    pub(crate) fn with_synchronized_span(&self, value: SynchronizedSpan) -> Self {
        Context {
            span: Some(Arc::new(value)),
            entries: self.entries.clone(),
            suppress_telemetry: self.suppress_telemetry,
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Context");

        #[cfg(feature = "trace")]
        let mut entries = self.entries.as_ref().map_or(0, |e| e.len());
        #[cfg(feature = "trace")]
        {
            if let Some(span) = &self.span {
                dbg.field("span", &span.span_context());
                entries += 1;
            } else {
                dbg.field("span", &"None");
            }
        }
        #[cfg(not(feature = "trace"))]
        let entries = self.entries.as_ref().map_or(0, |e| e.len());

        dbg.field("entries count", &entries)
            .field("suppress_telemetry", &self.suppress_telemetry)
            .finish()
    }
}

/// A guard that resets the current context to the prior context when dropped.
#[derive(Debug)]
pub struct ContextGuard {
    // The position of the context in the stack. This is used to pop the context.
    cx_pos: u16,
    // Ensure this type is !Send as it relies on thread locals
    _marker: PhantomData<*const ()>,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        let id = self.cx_pos;
        if id > ContextStack::BASE_POS && id < ContextStack::MAX_POS {
            CURRENT_CONTEXT.with(|context_stack| context_stack.borrow_mut().pop_id(id));
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

/// A stack for keeping track of the [`Context`] instances that have been attached
/// to a thread.
///
/// The stack allows for popping of contexts by position, which is used to do out
/// of order dropping of [`ContextGuard`] instances. Only when the top of the
/// stack is popped, the topmost [`Context`] is actually restored.
///
/// The stack relies on the fact that it is thread local and that the
/// [`ContextGuard`] instances that are constructed using ids from it can't be
/// moved to other threads. That means that the ids are always valid and that
/// they are always within the bounds of the stack.
struct ContextStack {
    /// This is the current [`Context`] that is active on this thread, and the top
    /// of the [`ContextStack`]. It is always present, and if the `stack` is empty
    /// it's an empty [`Context`].
    ///
    /// Having this here allows for fast access to the current [`Context`].
    current_cx: Context,
    /// A `stack` of the other contexts that have been attached to the thread.
    stack: Vec<Option<Context>>,
    /// Ensure this type is !Send as it relies on thread locals
    _marker: PhantomData<*const ()>,
}

impl ContextStack {
    const BASE_POS: u16 = 0;
    const MAX_POS: u16 = u16::MAX;
    const INITIAL_CAPACITY: usize = 8;

    #[inline(always)]
    fn push(&mut self, cx: Context) -> u16 {
        // The next id is the length of the `stack`, plus one since we have the
        // top of the [`ContextStack`] as the `current_cx`.
        let next_id = self.stack.len() + 1;
        if next_id < ContextStack::MAX_POS.into() {
            let current_cx = std::mem::replace(&mut self.current_cx, cx);
            self.stack.push(Some(current_cx));
            next_id as u16
        } else {
            // This is an overflow, log it and ignore it.
            otel_warn!(
                name: "Context.AttachFailed",
                message = format!("Too many contexts. Max limit is {}. \
                  Context::current() remains unchanged as this attach failed. \
                  Dropping the returned ContextGuard will have no impact on Context::current().",
                  ContextStack::MAX_POS)
            );
            ContextStack::MAX_POS
        }
    }

    #[inline(always)]
    fn pop_id(&mut self, pos: u16) {
        if pos == ContextStack::BASE_POS || pos == ContextStack::MAX_POS {
            // The empty context is always at the bottom of the [`ContextStack`]
            // and cannot be popped, and the overflow position is invalid, so do
            // nothing.
            otel_warn!(
                name: "Context.OutOfOrderDrop",
                position = pos,
                message = if pos == ContextStack::BASE_POS {
                    "Attempted to pop the base context which is not allowed"
                } else {
                    "Attempted to pop the overflow position which is not allowed"
                }
            );
            return;
        }
        let len: u16 = self.stack.len() as u16;
        // Are we at the top of the [`ContextStack`]?
        if pos == len {
            // Shrink the stack if possible to clear out any out of order pops.
            while let Some(None) = self.stack.last() {
                _ = self.stack.pop();
            }
            // Restore the previous context. This will always happen since the
            // empty context is always at the bottom of the stack if the
            // [`ContextStack`] is not empty.
            if let Some(Some(next_cx)) = self.stack.pop() {
                self.current_cx = next_cx;
            }
        } else {
            // This is an out of order pop.
            if pos >= len {
                // This is an invalid id, ignore it.
                otel_warn!(
                    name: "Context.PopOutOfBounds",
                    position = pos,
                    stack_length = len,
                    message = "Attempted to pop beyond the end of the context stack"
                );
                return;
            }
            // Clear out the entry at the given id.
            _ = self.stack[pos as usize].take();
        }
    }

    #[inline(always)]
    fn map_current_cx<T>(&self, f: impl FnOnce(&Context) -> T) -> T {
        f(&self.current_cx)
    }
}

impl Default for ContextStack {
    fn default() -> Self {
        ContextStack {
            current_cx: Context::default(),
            stack: Vec::with_capacity(ContextStack::INITIAL_CAPACITY),
            _marker: PhantomData,
        }
    }
}
