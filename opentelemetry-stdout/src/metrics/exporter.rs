use core::fmt;
use std::{
    io::{stdout, Write},
    sync::Mutex,
};

use async_trait::async_trait;
use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_sdk::metrics::{
    data,
    exporter::PushMetricsExporter,
    reader::{
        AggregationSelector, DefaultAggregationSelector, DefaultTemporalitySelector,
        TemporalitySelector,
    },
    Aggregation, InstrumentKind,
};

use crate::MetricsData;

type Encoder = Box<dyn Fn(&mut dyn Write, MetricsData) -> Result<()> + Send + Sync>;

/// An OpenTelemetry exporter that writes to stdout on export.
pub struct MetricsExporter {
    writer: Mutex<Option<Box<dyn Write + Send + Sync>>>,
    encoder: Encoder,
    temporality_selector: Box<dyn TemporalitySelector>,
    aggregation_selector: Box<dyn AggregationSelector>,
}

impl MetricsExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> MetricsExporterBuilder {
        MetricsExporterBuilder::default()
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        MetricsExporterBuilder::default().build()
    }
}

impl fmt::Debug for MetricsExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricsExporter")
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality(&self, kind: InstrumentKind) -> data::Temporality {
        self.temporality_selector.temporality(kind)
    }
}

impl AggregationSelector for MetricsExporter {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.aggregation_selector.aggregation(kind)
    }
}

#[async_trait]
impl PushMetricsExporter for MetricsExporter {
    async fn export(&self, metrics: &mut data::ResourceMetrics) -> Result<()> {
        if let Some(writer) = self.writer.lock()?.as_mut() {
            (self.encoder)(writer, crate::metrics::MetricsData::from(metrics))?;
            writer
                .write_all(b"\n")
                .map_err(|err| MetricsError::Other(err.to_string()))
        } else {
            Err(MetricsError::Other("exporter is shut down".into()))
        }
    }

    async fn force_flush(&self) -> Result<()> {
        // exporter holds no state, nothing to flush
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        self.writer.lock()?.take();
        Ok(())
    }
}

/// Configuration for the stdout metrics exporter
#[derive(Default)]
pub struct MetricsExporterBuilder {
    writer: Option<Box<dyn Write + Send + Sync>>,
    encoder: Option<Encoder>,
    temporality_selector: Option<Box<dyn TemporalitySelector>>,
    aggregation_selector: Option<Box<dyn AggregationSelector>>,
}

impl MetricsExporterBuilder {
    /// Set the writer that the exporter will write to
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry_stdout::MetricsExporterBuilder;
    ///
    /// let buffer = Vec::new(); // Any type that implements `Write`
    /// let exporter = MetricsExporterBuilder::default().with_writer(buffer).build();
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
    /// use opentelemetry_stdout::MetricsExporterBuilder;
    ///
    /// let exporter = MetricsExporterBuilder::default()
    ///     // Can be any function that can write formatted data
    ///     // serde ecosystem crates for example provide such functions
    ///     .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
    ///     .build();
    /// ```
    pub fn with_encoder(
        mut self,
        encoder: impl Fn(&mut dyn Write, MetricsData) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.encoder = Some(Box::new(encoder));
        self
    }

    /// Set the temporality exporter for the exporter
    pub fn with_temporality_selector(
        mut self,
        selector: impl TemporalitySelector + 'static,
    ) -> Self {
        self.temporality_selector = Some(Box::new(selector));
        self
    }

    /// Set the aggregation exporter for the exporter
    pub fn with_aggregation_selector(
        mut self,
        selector: impl AggregationSelector + 'static,
    ) -> Self {
        self.aggregation_selector = Some(Box::new(selector));
        self
    }

    /// Create a metrics exporter with the current configuration
    pub fn build(self) -> MetricsExporter {
        MetricsExporter {
            writer: Mutex::new(Some(self.writer.unwrap_or_else(|| Box::new(stdout())))),
            encoder: self.encoder.unwrap_or_else(|| {
                Box::new(|writer, metrics| {
                    serde_json::to_writer(writer, &metrics)
                        .map_err(|err| MetricsError::Other(err.to_string()))
                })
            }),
            temporality_selector: self
                .temporality_selector
                .unwrap_or_else(|| Box::new(DefaultTemporalitySelector::new())),
            aggregation_selector: self
                .aggregation_selector
                .unwrap_or_else(|| Box::new(DefaultAggregationSelector::new())),
        }
    }
}

impl fmt::Debug for MetricsExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricsExporterBuilder")
    }
}
