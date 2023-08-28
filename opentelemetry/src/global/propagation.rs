use crate::propagation::TextMapPropagator;
use crate::trace::noop::NoopTextMapPropagator;
use once_cell::sync::Lazy;
use std::sync::RwLock;

/// The current global `TextMapPropagator` propagator.
static GLOBAL_TEXT_MAP_PROPAGATOR: Lazy<RwLock<Box<dyn TextMapPropagator + Send + Sync>>> =
    Lazy::new(|| RwLock::new(Box::new(NoopTextMapPropagator::new())));

/// The global default `TextMapPropagator` propagator.
static DEFAULT_TEXT_MAP_PROPAGATOR: Lazy<NoopTextMapPropagator> =
    Lazy::new(NoopTextMapPropagator::new);

/// Sets the given [`TextMapPropagator`] propagator as the current global propagator.
pub fn set_text_map_propagator<P: TextMapPropagator + Send + Sync + 'static>(propagator: P) {
    let _lock = GLOBAL_TEXT_MAP_PROPAGATOR
        .write()
        .map(|mut global_propagator| *global_propagator = Box::new(propagator));
}

/// Executes a closure with a reference to the current global [`TextMapPropagator`] propagator.
pub fn get_text_map_propagator<T, F>(mut f: F) -> T
where
    F: FnMut(&dyn TextMapPropagator) -> T,
{
    GLOBAL_TEXT_MAP_PROPAGATOR
        .read()
        .map(|propagator| f(&**propagator))
        .unwrap_or_else(|_| f(&*DEFAULT_TEXT_MAP_PROPAGATOR as &dyn TextMapPropagator))
}
