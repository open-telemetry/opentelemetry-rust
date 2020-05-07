//! # Stdout Span Exporter
//!
//! The stdout [`SpanExporter`] writes debug printed [`Span`]s to its configured
//! [`Write`] instance. By default it will write to [`Stdout`].
//!
//! [`SpanExporter`]: ../trait.SpanExporter.html
//! [`Span`]: ../../../api/trace/span/trait.Span.html
//! [`Write`]: std::io::Write
//! [`Stdout`]: std::io::Stdout
//!
//! # Examples
//!
//! ```
//! use opentelemetry::exporter::trace::stdout;
//! use opentelemetry::{sdk, global};
//!
//! // Create a new stdout exporter that writes pretty printed span output
//! let exporter = stdout::Builder::default().with_pretty_print(true).init();
//! let provider = sdk::Provider::builder()
//!     .with_simple_exporter(exporter)
//!     .build();
//! global::set_provider(provider);
//! ```
use crate::exporter::trace;
use std::fmt::Debug;
use std::io::{self, stdout, Stdout, Write};
use std::sync::{Arc, Mutex};

/// Builder
#[derive(Debug)]
pub struct Builder<W: Write + Debug> {
    writer: Mutex<W>,
    pretty_print: bool,
}

impl<W: Write + Debug> Builder<W> {
    /// Specify the writer to use with this exporter
    pub fn with_writer<T: Write + Debug>(self, writer: T) -> Builder<T> {
        Builder {
            writer: Mutex::new(writer),
            pretty_print: self.pretty_print,
        }
    }

    /// Specify the pretty print setting for this exporter
    pub fn with_pretty_print(self, pretty_print: bool) -> Self {
        Builder {
            pretty_print,
            ..self
        }
    }

    /// Build a new exporter
    pub fn init(self) -> Exporter<W> {
        Exporter {
            writer: self.writer,
            pretty_print: self.pretty_print,
        }
    }
}

impl Default for Builder<Stdout> {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        Builder {
            writer: Mutex::new(stdout()),
            pretty_print: false,
        }
    }
}

/// A [`SpanExporter`] that writes to [`Stdout`] or other configured [`Write`].
///
/// [`SpanExporter`]: ../trait.SpanExporter.html
/// [`Write`]: std::io::Write
/// [`Stdout`]: std::io::Stdout
#[derive(Debug)]
pub struct Exporter<W: Write> {
    writer: Mutex<W>,
    pretty_print: bool,
}

impl<W> trace::SpanExporter for Exporter<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Export spans to stdout
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        let writer = self
            .writer
            .try_lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));
        let result = writer.and_then(|mut w| {
            for span in batch {
                if self.pretty_print {
                    w.write_all(format!("{:#?}\n", span).as_bytes())?;
                } else {
                    w.write_all(format!("{:?}\n", span).as_bytes())?;
                }
            }

            Ok(0)
        });

        if result.is_ok() {
            trace::ExportResult::Success
        } else {
            // FIXME: determine retryable io::Error types
            trace::ExportResult::FailedNotRetryable
        }
    }

    /// Ignored for now.
    fn shutdown(&self) {}
}
