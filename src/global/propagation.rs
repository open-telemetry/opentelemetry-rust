use crate::api;
use std::sync::RwLock;

lazy_static::lazy_static! {
    /// The current global `TextMapFormat` propagator.
    static ref GLOBAL_TEXT_MAP_PROPAGATOR: RwLock<Box<dyn api::TextMapFormat + Send + Sync>> = RwLock::new(Box::new(api::TextMapCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())])));
    /// The global default `TextMapFormat` propagator.
    static ref DEFAULT_TEXT_MAP_PROPAGATOR: api::TextMapCompositePropagator = api::TextMapCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())]);
}

/// Sets the given [`TextMapFormat`] propagator as the current global propagator.
///
/// [`TextMapFormat`]: ../api/context/propagation/trait.TextMapFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, global};
///
/// // create your text map propagator
/// let propagator = api::TraceContextPropagator::new();
///
/// // assign it as the global propagator
/// global::set_text_map_propagator(propagator);
/// ```
pub fn set_text_map_propagator<P: api::TextMapFormat + Send + Sync + 'static>(propagator: P) {
    let _lock = GLOBAL_TEXT_MAP_PROPAGATOR
        .write()
        .map(|mut global_propagator| *global_propagator = Box::new(propagator));
}

/// Executes a closure with a reference to the current global [`TextMapFormat`] propagator.
///
/// [`TextMapFormat`]: ../api/context/propagation/trait.TextMapFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, api::TextMapFormat, global};
/// use std::collections::HashMap;
///
/// let example_carrier = HashMap::new();
///
/// // create your text map propagator
/// let tc_propagator = api::TraceContextPropagator::new();
/// global::set_text_map_propagator(tc_propagator);
///
/// // use the global text map propagator to extract contexts
/// let _cx = global::get_text_map_propagator(|propagator| propagator.extract(&example_carrier));
/// ```
pub fn get_text_map_propagator<T, F>(mut f: F) -> T
where
    F: FnMut(&dyn api::TextMapFormat) -> T,
{
    GLOBAL_TEXT_MAP_PROPAGATOR
        .read()
        .map(|propagator| f(&**propagator))
        .unwrap_or_else(|_| f(&*DEFAULT_TEXT_MAP_PROPAGATOR as &dyn api::TextMapFormat))
}
