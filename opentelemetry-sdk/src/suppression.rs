use pin_project_lite::pin_project;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::thread;
use tokio::task;

thread_local! {
    static SUPPRESSION_ENABLED: RefCell<bool> = RefCell::new(false)
}

#[derive(Clone, Copy, Debug)]
pub struct SuppressionContext;

impl SuppressionContext {
    pub fn current() -> Self {
        SuppressionContext
    }

    pub fn attach(self) -> Guard {
        let was_suppressed = SUPPRESSION_ENABLED.with(|suppressed| {
            let was_suppressed = *suppressed.borrow();
            *suppressed.borrow_mut() = true;
            was_suppressed
        });
        Guard { was_suppressed }
    }

    pub fn is_suppressed() -> bool {
        let is_suppressed = SUPPRESSION_ENABLED.with(|suppressed| *suppressed.borrow());
        is_suppressed
    }
}

pub struct Guard {
    was_suppressed: bool,
}

impl Drop for Guard {
    fn drop(&mut self) {
        SUPPRESSION_ENABLED.with(|suppressed| *suppressed.borrow_mut() = self.was_suppressed);
    }
}

pin_project! {
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

pub fn with_suppression<T>(inner: T) -> WithSuppContext<T> {
    WithSuppContext {
        inner,
        supp_cx: SuppressionContext::current(),
    }
}

pub fn is_suppressed() -> bool {
    SuppressionContext::is_suppressed()
}
