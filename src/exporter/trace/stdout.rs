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
//! use opentelemetry::api::Tracer;
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
use crate::{api::TracerProvider, exporter::trace, global, sdk};
use async_trait::async_trait;
use std::fmt::Debug;
use std::io::{self, stdout, Stdout, Write};
use std::sync::{Arc, Mutex};

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
    writer: Mutex<W>,
    pretty_print: bool,
}

impl<W: Write> Exporter<W> {
    /// Create a new stdout `Exporter`.
    pub fn new(writer: W, pretty_print: bool) -> Self {
        Self {
            writer: Mutex::new(writer),
            pretty_print,
        }
    }
}

#[async_trait]
impl<W> trace::SpanExporter for Exporter<W>
where
    W: Write + Debug + Send + 'static,
{
    /// Export spans to stdout
    async fn export(&self, batch: &[Arc<trace::SpanData>]) -> trace::ExportResult {
        let writer = self
            .writer
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));
        let result = writer.and_then(|mut w| {
            for span in batch {
                if self.pretty_print {
                    w.write_all(format!("{:#?}\n", span).as_bytes())?;
                } else {
                    w.write_all(format!("{:?}\n", span).as_bytes())?;
                }
            }

            Ok(())
        });

        if result.is_ok() {
            trace::ExportResult::Success
        } else {
            // FIXME: determine retryable io::Error types
            trace::ExportResult::FailedNotRetryable
        }
    }
}

/// Uninstalls the stdout pipeline on drop.
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);
