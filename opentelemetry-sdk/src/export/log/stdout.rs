//! # Stdout Log Exporter
//!
//! The stdout [`LogExporter`] writes debug printed [`LogRecord`]s to its configured
//! [`Write`] instance. By default it will write to [`Stdout`].
//!
//! [`LogExporter`]: super::LogExporter
//! [`LogRecord`]: crate::log::LogRecord
//! [`Write`]: std::io::Write
//! [`Stdout`]: std::io::Stdout
//!
//! # Examples
//!
//! ```no_run
//! use opentelemetry_api::global::shutdown_tracer_provider;
//! use opentelemetry_api::trace::Tracer;
//! use opentelemetry_sdk::export::trace::stdout;
//!
//! fn main() {
//!     let tracer = stdout::new_pipeline()
//!         .with_pretty_print(true)
//!         .install_simple();
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     shutdown_tracer_provider(); // sending remaining spans
//! }
//! ```
use crate::export::{
    log::{ExportResult, LogData, LogExporter},
    ExportError,
};
use async_trait::async_trait;
use opentelemetry_api::log::LogError;
use std::fmt::Debug;
use std::io::{stdout, Stdout, Write};

/// Pipeline builder
#[derive(Debug)]
pub struct PipelineBuilder<W: Write> {
    pretty_print: bool,
    log_config: Option<crate::log::Config>,
    writer: W,
}

/// Create a new stdout exporter pipeline builder.
pub fn new_pipeline() -> PipelineBuilder<Stdout> {
    PipelineBuilder::default()
}

impl Default for PipelineBuilder<Stdout> {
    /// Return the default pipeline builder.
    fn default() -> Self {
        Self {
            pretty_print: false,
            log_config: None,
            writer: stdout(),
        }
    }
}

impl<W: Write> PipelineBuilder<W> {
    /// Specify the pretty print setting.
    pub fn with_pretty_print(mut self, pretty_print: bool) -> Self {
        self.pretty_print = pretty_print;
        self
    }

    /// Assign the SDK trace configuration.
    pub fn with_trace_config(mut self, config: crate::log::Config) -> Self {
        self.log_config = Some(config);
        self
    }

    /// Specify the writer to use.
    pub fn with_writer<T: Write>(self, writer: T) -> PipelineBuilder<T> {
        PipelineBuilder {
            pretty_print: self.pretty_print,
            log_config: self.log_config,
            writer,
        }
    }
}

impl<W> PipelineBuilder<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Install the stdout exporter pipeline with the recommended defaults.
    pub fn install_simple(mut self) -> crate::log::LogEmitter {
        let exporter = Exporter::new(self.writer, self.pretty_print);

        let mut provider_builder =
            crate::log::LogEmitterProvider::builder().with_simple_exporter(exporter);
        if let Some(config) = self.log_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();

        provider.versioned_log_emitter("opentelemetry", Some(env!("CARGO_PKG_VERSION")))
    }
}

/// A [`LogExporter`] that writes to [`Stdout`] or other configured [`Write`].
///
/// [`LogExporter`]: super::LogExporter
/// [`Write`]: std::io::Write
/// [`Stdout`]: std::io::Stdout
#[derive(Debug)]
pub struct Exporter<W: Write> {
    writer: W,
    pretty_print: bool,
}

impl<W: Write> Exporter<W> {
    /// Create a new stdout `Exporter`.
    pub fn new(writer: W, pretty_print: bool) -> Self {
        Self {
            writer,
            pretty_print,
        }
    }
}

#[async_trait]
impl<W> LogExporter for Exporter<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Export spans to stdout
    async fn export(&mut self, batch: Vec<LogData>) -> ExportResult {
        for log in batch {
            if self.pretty_print {
                self.writer
                    .write_all(format!("{:#?}\n", log).as_bytes())
                    .map_err(|err| LogError::ExportFailed(Box::new(Error::from(err))))?;
            } else {
                self.writer
                    .write_all(format!("{:?}\n", log).as_bytes())
                    .map_err(|err| LogError::ExportFailed(Box::new(Error::from(err))))?;
            }
        }

        Ok(())
    }
}

/// Stdout exporter's error
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
struct Error(#[from] std::io::Error);

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "stdout"
    }
}
