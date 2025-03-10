use crate::Context;
use crate::trace::WithContext;

/// Extension trait allowing futures, streams, and sinks to be traced with a span.
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as current while it is being polled.
    ///
    /// [`Context`]: Context
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
    /// [`Context`]: Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}
