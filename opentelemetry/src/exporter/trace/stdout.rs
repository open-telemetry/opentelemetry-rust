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
//! ```no_run
//! use opentelemetry::trace::Tracer;
//! use opentelemetry::exporter::trace::stdout;
//!
//! fn main() {
//!     let (tracer, _uninstall) = stdout::new_pipeline()
//!         .with_pretty_print(true)
//!         .install();
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//! }
//! ```
use crate::exporter::trace::ExportError;
use crate::{
    exporter::trace::{ExportResult, SpanData, SpanExporter},
    global, sdk,
    trace::TracerProvider,
};
use async_trait::async_trait;
use serde::export::Formatter;
use std::fmt::{Debug, Display};
use std::io::{stdout, Stdout, Write};

/// Pipeline builder
#[derive(Debug)]
pub struct PipelineBuilder<W: Write> {
    pretty_print: bool,
    trace_config: Option<sdk::trace::Config>,
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
            trace_config: None,
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
    pub fn with_trace_config(mut self, config: sdk::trace::Config) -> Self {
        self.trace_config = Some(config);
        self
    }

    /// Specify the writer to use.
    pub fn with_writer<T: Write>(self, writer: T) -> PipelineBuilder<T> {
        PipelineBuilder {
            pretty_print: self.pretty_print,
            trace_config: self.trace_config,
            writer,
        }
    }
}

impl<W> PipelineBuilder<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Install the stdout exporter pipeline with the recommended defaults.
    pub fn install(mut self) -> (sdk::trace::Tracer, Uninstall) {
        let exporter = Exporter::new(self.writer, self.pretty_print);

        let mut provider_builder = sdk::trace::TracerProvider::builder().with_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();
        let tracer = provider.get_tracer("opentelemetry", Some(env!("CARGO_PKG_VERSION")));
        let provider_guard = global::set_tracer_provider(provider);

        (tracer, Uninstall(provider_guard))
    }
}

/// A [`SpanExporter`] that writes to [`Stdout`] or other configured [`Write`].
///
/// [`SpanExporter`]: ../trait.SpanExporter.html
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
impl<W> SpanExporter for Exporter<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Export spans to stdout
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        for span in batch {
            if self.pretty_print {
                self.writer
                    .write_all(format!("{:#?}\n", span).as_bytes())
                    .map_err(|err| Error::from(err))?;
            } else {
                self.writer
                    .write_all(format!("{:?}\n", span).as_bytes())
                    .map_err(|err| Error::from(err))?;
            }
        }

        Ok(())
    }
}

/// Uninstalls the stdout pipeline on dropping.
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);

/// Stdout exporter's error
#[derive(Debug)]
struct Error(std::io::Error);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(err)
    }
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "stdout"
    }
}
