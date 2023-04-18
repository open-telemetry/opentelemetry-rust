//! # Stdout Span Exporter
//!
//! The stdout [`SpanExporter`] writes debug printed [`Span`]s to its configured
//! [`Write`] instance. By default it will write to [`Stdout`].
//!
//! [`SpanExporter`]: super::SpanExporter
//! [`Span`]: crate::trace::Span
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
    trace::{ExportResult, SpanData, SpanExporter, TraceError},
    ExportError,
};
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use opentelemetry_api::{global, trace::TracerProvider};
use std::fmt::Debug;
use std::io::{stdout, Stdout, Write};

/// Pipeline builder
#[derive(Debug)]
pub struct PipelineBuilder<W: Write> {
    pretty_print: bool,
    trace_config: Option<crate::trace::Config>,
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
    pub fn with_trace_config(mut self, config: crate::trace::Config) -> Self {
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
    pub fn install_simple(mut self) -> crate::trace::Tracer {
        let exporter = Exporter::new(self.writer, self.pretty_print);

        let mut provider_builder =
            crate::trace::TracerProvider::builder().with_simple_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();

        let tracer = provider.versioned_tracer(
            "opentelemetry".into(),
            Some(env!("CARGO_PKG_VERSION").into()),
            None,
            None,
        );
        let _ = global::set_tracer_provider(provider);

        tracer
    }

    /// Install the stdout exporter pipeline with the recommended defaults and specific attributes
    pub fn install_with_tracer_attributes(
        mut self,
        attributes: Vec<opentelemetry_api::KeyValue>,
    ) -> crate::trace::Tracer {
        let exporter = Exporter::new(self.writer, self.pretty_print);

        let mut provider_builder =
            crate::trace::TracerProvider::builder().with_simple_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();

        let tracer = provider.versioned_tracer(
            "opentelemetry",
            Some(env!("CARGO_PKG_VERSION")),
            None,
            Some(attributes),
        );
        let _ = global::set_tracer_provider(provider);

        tracer
    }

}

/// A [`SpanExporter`] that writes to [`Stdout`] or other configured [`Write`].
///
/// [`SpanExporter`]: super::SpanExporter
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
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        for span in batch {
            if self.pretty_print {
                if let Err(err) = self
                    .writer
                    .write_all(format!("{:#?}\n", span).as_bytes())
                    .map_err(|err| TraceError::ExportFailed(Box::new(Error::from(err))))
                {
                    return Box::pin(std::future::ready(Err(Into::into(err))));
                }
            } else if let Err(err) = self
                .writer
                .write_all(format!("{:?}\n", span).as_bytes())
                .map_err(|err| TraceError::ExportFailed(Box::new(Error::from(err))))
            {
                return Box::pin(std::future::ready(Err(Into::into(err))));
            }
        }

        Box::pin(std::future::ready(Ok(())))
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
