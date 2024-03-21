use async_trait::async_trait;
use core::fmt;
use opentelemetry::{
    logs::{LogError, LogResult},
    ExportError,
};
use opentelemetry_sdk::export::logs::{ExportResult, LogEvent};
use opentelemetry_sdk::Resource;
use std::io::{stdout, Write};

type Encoder =
    Box<dyn Fn(&mut dyn Write, crate::logs::transform::LogData) -> LogResult<()> + Send + Sync>;

/// A [`LogExporter`] that writes to [`Stdout`] or other configured [`Write`].
///
/// [`LogExporter`]: opentelemetry_sdk::export::logs::LogExporter
/// [`Write`]: std::io::Write
/// [`Stdout`]: std::io::Stdout
pub struct LogExporter {
    writer: Option<Box<dyn Write + Send + Sync>>,
    encoder: Encoder,
    resource: Resource,
}

impl LogExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> LogExporterBuilder {
        Default::default()
    }
}

impl Default for LogExporter {
    fn default() -> Self {
        LogExporterBuilder::default().build()
    }
}

impl fmt::Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LogsExporter")
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    /// Export spans to stdout
    async fn export(&mut self, batch: Vec<LogEvent>) -> ExportResult {
        if let Some(writer) = &mut self.writer {
            let log_data = crate::logs::transform::LogData::from((batch, &self.resource));
            let result = (self.encoder)(writer, log_data) as LogResult<()>;
            result.and_then(|_| writer.write_all(b"\n").map_err(|e| Error(e).into()))
        } else {
            Err("exporter is shut down".into())
        }
    }

    fn shutdown(&mut self) {
        self.writer.take();
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
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

/// Configuration for the stdout log exporter
#[derive(Default)]
pub struct LogExporterBuilder {
    writer: Option<Box<dyn Write + Send + Sync>>,
    encoder: Option<Encoder>,
}

impl fmt::Debug for LogExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LogExporterBuilder")
    }
}

impl LogExporterBuilder {
    /// Set the writer that the exporter will write to
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_stdout::LogExporterBuilder;
    ///
    /// let buffer = Vec::new(); // Any type that implements `Write`
    /// let exporter = LogExporterBuilder::default().with_writer(buffer).build();
    /// ```
    pub fn with_writer<W>(mut self, writer: W) -> Self
    where
        W: Write + Send + Sync + 'static,
    {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Set the encoder that the exporter will use.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_stdout::LogExporterBuilder;
    /// use serde_json;
    ///
    /// let exporter = LogExporterBuilder::default()
    ///     .with_encoder(|writer, data|
    ///          Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
    ///     .build();
    /// ```
    pub fn with_encoder<E>(mut self, encoder: E) -> Self
    where
        E: Fn(&mut dyn Write, crate::logs::transform::LogData) -> LogResult<()>
            + Send
            + Sync
            + 'static,
    {
        self.encoder = Some(Box::new(encoder));
        self
    }

    /// Create a log exporter with the current configuration.
    pub fn build(self) -> LogExporter {
        LogExporter {
            writer: Some(self.writer.unwrap_or_else(|| Box::new(stdout()))),
            resource: Resource::default(),
            encoder: self.encoder.unwrap_or_else(|| {
                Box::new(|writer, logs| {
                    serde_json::to_writer(writer, &logs)
                        .map_err(|err| LogError::Other(Box::new(err)))
                })
            }),
        }
    }
}
