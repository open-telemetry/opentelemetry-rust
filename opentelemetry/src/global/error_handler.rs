use crate::api::Error;
use std::sync::RwLock;

lazy_static::lazy_static! {
    /// The global error handler.
    static ref GLOBAL_ERROR_HANDLER: RwLock<Option<ErrorHandler>> = RwLock::new(None);
}

struct ErrorHandler(Box<dyn Fn(Error) + Send + Sync>);

/// Handle error using the globally configured error handler.
///
/// Writes to stderr if unset.
pub fn handle_error<T: Into<Error>>(err: T) {
    match GLOBAL_ERROR_HANDLER.read() {
        Ok(handler) if handler.is_some() => (handler.as_ref().unwrap().0)(err.into()),
        _ => match err.into() {
            #[cfg(feature = "metrics")]
            #[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
            Error::Metric(err) => eprintln!("OpenTelemetry metrics error occurred {:?}", err),
            #[cfg(feature = "trace")]
            #[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
            Error::Trace(err) => eprintln!("OpenTelemetry trace error occurred {:?}", err),
            Error::Other(err_msg) => println!("OpenTelemetry error occurred {}", err_msg),
        },
    }
}

/// Set global error handler.
pub fn set_error_handler<F>(f: F) -> std::result::Result<(), Error>
where
    F: Fn(Error) + Send + Sync + 'static,
{
    GLOBAL_ERROR_HANDLER
        .write()
        .map(|mut handler| *handler = Some(ErrorHandler(Box::new(f))))
        .map_err(Into::into)
}
