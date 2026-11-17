use crate::propagation::TextMapPropagator;
use crate::trace::noop::NoopTextMapPropagator;
use std::ops::Deref;
use std::sync::Arc;

use arc_swap::ArcSwapOption;

type GlobalTextMapPropagator = Box<dyn TextMapPropagator + Send + Sync>;

/// The global `TextMapPropagator` singleton, lazily initialized with a no-op propagator.
static GLOBAL_TEXT_MAP_PROPAGATOR: ArcSwapOption<GlobalTextMapPropagator> =
    ArcSwapOption::const_empty();

/// Sets the given [`TextMapPropagator`] propagator as the current global propagator.
pub fn set_text_map_propagator<P: TextMapPropagator + Send + Sync + 'static>(propagator: P) {
    GLOBAL_TEXT_MAP_PROPAGATOR.store(Some(Arc::new(Box::new(propagator))));
}

/// Executes a closure with a reference to the current global [`TextMapPropagator`] propagator.
pub fn get_text_map_propagator<T, F>(mut f: F) -> T
where
    F: FnMut(&dyn TextMapPropagator) -> T,
{
    let current = GLOBAL_TEXT_MAP_PROPAGATOR.load();
    if let Some(propagator) = current.as_deref() {
        return f(propagator.as_ref());
    }
    drop(current);

    // Lazily install the no-op propagator without overwriting a concurrent set.
    let noop = Arc::new(Box::new(NoopTextMapPropagator::new()) as GlobalTextMapPropagator);
    let previous = GLOBAL_TEXT_MAP_PROPAGATOR.compare_and_swap(&None::<Arc<_>>, Some(noop.clone()));
    match previous.as_deref() {
        Some(winner) => f(winner.as_ref()),
        None => f(noop.deref().as_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propagation::{text_map_propagator::FieldIter, Extractor, Injector};
    use crate::Context;

    #[derive(Debug)]
    struct TestPropagator {
        fields: Vec<String>,
    }

    impl TextMapPropagator for TestPropagator {
        fn inject_context(&self, _: &Context, _: &mut dyn Injector) {}

        fn extract_with_context(&self, cx: &Context, _: &dyn Extractor) -> Context {
            cx.clone()
        }

        fn fields(&self) -> FieldIter<'_> {
            FieldIter::new(&self.fields)
        }
    }

    #[test]
    fn test_global_text_map_propagator() {
        assert_eq!(
            get_text_map_propagator(|propagator| propagator.fields().count()),
            0
        );
        set_text_map_propagator(TestPropagator {
            fields: vec!["first".into()],
        });
        get_text_map_propagator(|propagator| {
            let current = GLOBAL_TEXT_MAP_PROPAGATOR.load();
            assert_eq!(Arc::strong_count(current.as_ref().unwrap()), 1);
            set_text_map_propagator(TestPropagator {
                fields: vec!["second".into()],
            });
            assert_eq!(propagator.fields().collect::<Vec<_>>(), vec!["first"]);
        });
        assert!(std::panic::catch_unwind(|| {
            get_text_map_propagator(|_| panic!("callback panic"));
        })
        .is_err());
        get_text_map_propagator(|propagator| {
            assert_eq!(propagator.fields().collect::<Vec<_>>(), vec!["second"]);
        });
        set_text_map_propagator(NoopTextMapPropagator::new());
    }
}
