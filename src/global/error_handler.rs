use crate::api::metrics::{MetricsError, Result};
use std::sync::RwLock;

lazy_static::lazy_static! {
    /// The global error handler.
    static ref GLOBAL_ERROR_HANDLER: RwLock<Option<ErrorHandler>> = RwLock::new(None);
}

struct ErrorHandler(Box<dyn Fn(MetricsError) + Send + Sync>);

/// Handle error using the globally configured error handler.
///
/// Writes to stderr if unset.
pub fn handle_error(err: MetricsError) {
    match GLOBAL_ERROR_HANDLER.read() {
        Ok(handler) if handler.is_some() => (handler.as_ref().unwrap().0)(err),
        _ => eprintln!("OpenTelemetry metrics error occurred {:?}", err),
    }
}

/// Set global error handler.
pub fn set_error_handler<F>(f: F) -> Result<()>
where
    F: Fn(MetricsError) + Send + Sync + 'static,
{
    GLOBAL_ERROR_HANDLER
        .write()
        .map(|mut handler| *handler = Some(ErrorHandler(Box::new(f))))
        .map_err(Into::into)
}
