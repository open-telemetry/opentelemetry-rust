use pin_project_lite::pin_project;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tokio::task_local;

task_local! {
    static SUPPRESSION_ENABLED: RefCell<bool>
}

#[derive(Clone, Copy, Debug)]
///suppression context
pub struct SuppressionContext;

impl SuppressionContext {
    ///current
    pub fn current() -> Self {
        SuppressionContext
    }

    ///attach
    pub fn attach(self) -> Guard {
        let was_suppressed = SUPPRESSION_ENABLED
            .try_with(|suppressed| {
                let was_suppressed = *suppressed.borrow();
                *suppressed.borrow_mut() = true;
                was_suppressed
            })
            .unwrap_or_else(|_| {
                SUPPRESSION_ENABLED.with(|suppressed| {
                    *suppressed.borrow_mut() = true;
                    false // was_suppressed is false since it was not previously initialized
                })
            });
        Guard { was_suppressed }
    }
    ///is_suppressed
    pub fn is_suppressed() -> bool {
        SUPPRESSION_ENABLED
            .try_with(|suppressed| *suppressed.borrow())
            .unwrap_or(false)
    }
}

/// Guard
#[derive(Debug)]
pub struct Guard {
    was_suppressed: bool,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = SUPPRESSION_ENABLED.try_with(|suppressed| {
            *suppressed.borrow_mut() = self.was_suppressed;
        });
    }
}

pin_project! {
    /// WithSuppContext
    #[derive(Clone, Debug)]
    pub struct WithSuppContext<T> {
        #[pin]
        inner: T,
        supp_cx: SuppressionContext,
    }
}

impl<T: Future> Future for WithSuppContext<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.supp_cx.attach();
        this.inner.poll(task_cx)
    }
}

/// with_init_suppression
pub async fn with_init_suppression<F, Fut>(func: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    SUPPRESSION_ENABLED
        .scope(RefCell::new(true), async {
            func().await;
        })
        .await;
}

/// with_suppression
pub fn with_suppression<T>(inner: T) -> WithSuppContext<T> {
    WithSuppContext {
        inner,
        supp_cx: SuppressionContext::current(),
    }
}

/// is_suppressed
pub fn is_suppressed() -> bool {
    SuppressionContext::is_suppressed()
}
