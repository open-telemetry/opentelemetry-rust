use core::fmt;
use std::io::{stdout, Write};

use futures_util::future::BoxFuture;
use opentelemetry_api::trace::{TraceError, TraceResult};
use opentelemetry_sdk::export::{self, trace::ExportResult};

use crate::trace::transform::SpanData;

type Encoder = Box<dyn Fn(&mut dyn Write, SpanData) -> TraceResult<()> + Send + Sync>;

/// An OpenTelemetry exporter that writes to stdout on export.
pub struct SpanExporter {
    writer: Option<Box<dyn Write + Send + Sync>>,
    encoder: Encoder,
}

impl fmt::Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SpanExporter")
    }
}

impl SpanExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> SpanExporterBuilder {
        SpanExporterBuilder::default()
    }
}

impl Default for SpanExporter {
    fn default() -> Self {
        SpanExporterBuilder::default().build()
    }
}

impl opentelemetry_sdk::export::trace::SpanExporter for SpanExporter {
    fn export(&mut self, batch: Vec<export::trace::SpanData>) -> BoxFuture<'static, ExportResult> {
        let res = if let Some(writer) = &mut self.writer {
            (self.encoder)(writer, crate::trace::SpanData::from(batch)).and_then(|_| {
                writer
                    .write_all(b"\n")
                    .map_err(|err| TraceError::Other(Box::new(err)))
            })
        } else {
            Err("exporter is shut down".into())
        };

        Box::pin(std::future::ready(res))
    }

    fn shutdown(&mut self) {
        self.writer.take();
    }
}

/// Configuration for the stdout trace exporter
#[derive(Default)]
pub struct SpanExporterBuilder {
    writer: Option<Box<dyn Write + Send + Sync>>,
    encoder: Option<Encoder>,
}

impl fmt::Debug for SpanExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SpanExporterBuilder")
    }
}

impl SpanExporterBuilder {
    /// Set the writer that the exporter will write to
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_stdout::SpanExporterBuilder;
    ///
    /// let buffer = Vec::new(); // Any type that implements `Write`
    /// let exporter = SpanExporterBuilder::default().with_writer(buffer).build();
    /// ```
    pub fn with_writer(mut self, writer: impl Write + Send + Sync + 'static) -> Self {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Set the encoder that this exporter will use
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_stdout::SpanExporterBuilder;
    ///
    /// let exporter = SpanExporterBuilder::default()
    ///     // Can be any function that can write formatted data
    ///     // serde ecosystem crates for example provide such functions
    ///     .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
    ///     .build();
    /// ```
    pub fn with_encoder(
        mut self,
        writer: impl Fn(&mut dyn Write, SpanData) -> TraceResult<()> + Send + Sync + 'static,
    ) -> Self {
        self.encoder = Some(Box::new(writer));
        self
    }

    /// Create a span exporter with the current configuration
    pub fn build(self) -> SpanExporter {
        SpanExporter {
            writer: Some(self.writer.unwrap_or_else(|| Box::new(stdout()))),
            encoder: self.encoder.unwrap_or_else(|| {
                Box::new(|writer, spans| {
                    serde_json::to_writer(writer, &spans)
                        .map_err(|err| TraceError::Other(Box::new(err)))
                })
            }),
        }
    }
}
