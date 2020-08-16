use crate::api;
use std::sync::RwLock;

lazy_static::lazy_static! {
    /// The current global `HttpTextFormat` propagator.
    static ref GLOBAL_HTTP_TEXT_PROPAGATOR: RwLock<Box<dyn api::HttpTextFormat + Send + Sync>> = RwLock::new(Box::new(api::HttpTextCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())])));
    /// The global default `HttpTextFormat` propagator.
    static ref DEFAULT_HTTP_TEXT_PROPAGATOR: api::HttpTextCompositePropagator = api::HttpTextCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())]);
}

/// Sets the given [`HttpTextFormat`] propagator as the current global propagator.
///
/// [`HttpTextFormat`]: ../api/context/propagation/trait.HttpTextFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, global};
///
/// // create your http text propagator
/// let propagator = api::TraceContextPropagator::new();
///
/// // assign it as the global propagator
/// global::set_http_text_propagator(propagator);
/// ```
pub fn set_http_text_propagator<P: api::HttpTextFormat + Send + Sync + 'static>(propagator: P) {
    let _lock = GLOBAL_HTTP_TEXT_PROPAGATOR
        .write()
        .map(|mut global_propagator| *global_propagator = Box::new(propagator));
}

/// Executes a closure with a reference to the current global [`HttpTextFormat`] propagator.
///
/// [`HttpTextFormat`]: ../api/context/propagation/trait.HttpTextFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, api::HttpTextFormat, global};
/// use std::collections::HashMap;
///
/// let example_carrier = HashMap::new();
///
/// // create your http text propagator
/// let tc_propagator = api::TraceContextPropagator::new();
/// global::set_http_text_propagator(tc_propagator);
///
/// // use the global http text propagator to extract contexts
/// let _cx = global::get_http_text_propagator(|propagator| propagator.extract(&example_carrier));
/// ```
pub fn get_http_text_propagator<T, F>(mut f: F) -> T
where
    F: FnMut(&dyn api::HttpTextFormat) -> T,
{
    GLOBAL_HTTP_TEXT_PROPAGATOR
        .read()
        .map(|propagator| f(&**propagator))
        .unwrap_or_else(|_| f(&*DEFAULT_HTTP_TEXT_PROPAGATOR as &dyn api::HttpTextFormat))
}
