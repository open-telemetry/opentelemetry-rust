use core::fmt;
use std::io::{stdout, Write};

use async_trait::async_trait;
use opentelemetry_api::{
    logs::{LogError, LogResult},
    ExportError,
};
use opentelemetry_sdk::export::logs::{ExportResult, LogData};

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
}

impl LogExporter {
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
    async fn export(&mut self, batch: Vec<LogData>) -> ExportResult {
        let res = if let Some(writer) = &mut self.writer {
            (self.encoder)(writer, crate::logs::LogData::from(batch))
                .and_then(|_| writer.write_all(b"\n").map_err(|e| Error(e).into()))
        } else {
            Err("exporter is shut down".into())
        };

        res
    }

    fn shutdown(&mut self) {
        self.writer.take();
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
    pub fn with_writer<W>(mut self, writer: W) -> Self
    where
        W: Write + Send + Sync + 'static,
    {
        self.writer = Some(Box::new(writer));
        self
    }

    pub fn with_exporter<E>(mut self, encoder: E) -> Self
    where
        E: Fn(&mut dyn Write, crate::logs::transform::LogData) -> LogResult<()>
            + Send
            + Sync
            + 'static,
    {
        self.encoder = Some(Box::new(encoder));
        self
    }

    pub fn build(self) -> LogExporter {
        LogExporter {
            writer: Some(self.writer.unwrap_or_else(|| Box::new(stdout()))),
            encoder: self.encoder.unwrap_or_else(|| {
                Box::new(|writer, logs| {
                    serde_json::to_writer(writer, &logs)
                        .map_err(|err| LogError::Other(Box::new(err)))
                })
            }),
        }
    }
}
