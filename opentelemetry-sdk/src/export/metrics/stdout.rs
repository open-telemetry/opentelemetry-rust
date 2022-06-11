//! Stdout Metrics Exporter
use crate::{
    export::metrics::{
        aggregation::{stateless_temporality_selector, LastValue, Sum, TemporalitySelector},
        InstrumentationLibraryReader, MetricsExporter,
    },
    metrics::aggregators::{LastValueAggregator, SumAggregator},
    Resource,
};
use opentelemetry_api::{
    attributes::{default_encoder, AttributeSet, Encoder},
    metrics::{MetricsError, Result},
    Context, KeyValue,
};
use std::fmt;
use std::io;
use std::sync::Mutex;
use std::time::SystemTime;

/// Create a new stdout exporter builder with the configuration for a stdout exporter.
pub fn stdout() -> StdoutExporterBuilder<io::Stdout> {
    StdoutExporterBuilder::<io::Stdout>::builder()
}

/// An OpenTelemetry metric exporter that transmits telemetry to
/// the local STDOUT or via the registered implementation of `Write`.
#[derive(Debug)]
pub struct StdoutExporter<W> {
    /// Writer is the destination. If not set, `Stdout` is used.
    writer: Mutex<W>,

    /// Specifies if timestamps should be printed
    timestamps: bool,

    /// Encodes the attributes.
    attribute_encoder: Box<dyn Encoder + Send + Sync>,

    /// An optional user-defined function to format a given export batch.
    formatter: Option<Formatter>,
}

/// Individually exported metric
///
/// Can be formatted using [`StdoutExporterBuilder::with_formatter`].
#[derive(Default, Debug)]
pub struct ExportLine {
    /// metric name
    pub name: String,

    /// populated if using sum aggregator
    pub sum: Option<ExportNumeric>,

    /// populated if using last value aggregator
    pub last_value: Option<ExportNumeric>,

    /// metric timestamp
    pub timestamp: Option<SystemTime>,
}

/// A number exported as debug for serialization
pub struct ExportNumeric(Box<dyn fmt::Debug>);

impl fmt::Debug for ExportNumeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<W> StdoutExporter<W> {
    /// The temporality selector for this exporter
    pub fn temporality_selector(&self) -> impl TemporalitySelector {
        stateless_temporality_selector()
    }
}

impl<W> TemporalitySelector for StdoutExporter<W> {
    fn temporality_for(
        &self,
        descriptor: &crate::metrics::sdk_api::Descriptor,
        kind: &super::aggregation::AggregationKind,
    ) -> super::aggregation::Temporality {
        stateless_temporality_selector().temporality_for(descriptor, kind)
    }
}

impl<W> MetricsExporter for StdoutExporter<W>
where
    W: fmt::Debug + io::Write,
{
    fn export(
        &self,
        _cx: &Context,
        res: &Resource,
        reader: &dyn InstrumentationLibraryReader,
    ) -> Result<()> {
        let mut batch = Vec::new();
        reader.try_for_each(&mut |library, reader| {
            let mut attributes = Vec::new();
            if !library.name.is_empty() {
                attributes.push(KeyValue::new("instrumentation.name", library.name.clone()));
            }
            if let Some(version) = &library.version {
                attributes.push(KeyValue::new("instrumentation.version", version.clone()));
            }
            if let Some(schema) = &library.schema_url {
                attributes.push(KeyValue::new("instrumentation.schema_url", schema.clone()));
            }
            let inst_attributes = AttributeSet::from_attributes(attributes.into_iter());
            let encoded_inst_attributes =
                inst_attributes.encoded(Some(self.attribute_encoder.as_ref()));

            reader.try_for_each(self, &mut |record| {
                let desc = record.descriptor();
                let agg = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
                let kind = desc.number_kind();

                let encoded_resource = res.encoded(self.attribute_encoder.as_ref());

                let mut expose = ExportLine::default();
                if let Some(sum) = agg.as_any().downcast_ref::<SumAggregator>() {
                    expose.sum = Some(ExportNumeric(sum.sum()?.to_debug(kind)));
                } else if let Some(last_value) = agg.as_any().downcast_ref::<LastValueAggregator>()
                {
                    let (value, timestamp) = last_value.last_value()?;
                    expose.last_value = Some(ExportNumeric(value.to_debug(kind)));

                    if self.timestamps {
                        expose.timestamp = Some(timestamp);
                    }
                }

                let mut encoded_attributes = String::new();
                let iter = record.attributes().iter();
                if let (0, _) = iter.size_hint() {
                    encoded_attributes = record
                        .attributes()
                        .encoded(Some(self.attribute_encoder.as_ref()));
                }

                let mut sb = String::new();

                sb.push_str(desc.name());

                if !encoded_attributes.is_empty()
                    || !encoded_resource.is_empty()
                    || !encoded_inst_attributes.is_empty()
                {
                    sb.push('{');
                    sb.push_str(&encoded_resource);
                    if !encoded_inst_attributes.is_empty() && !encoded_resource.is_empty() {
                        sb.push(',');
                    }
                    sb.push_str(&encoded_inst_attributes);
                    if !encoded_attributes.is_empty()
                        && (!encoded_inst_attributes.is_empty() || !encoded_resource.is_empty())
                    {
                        sb.push(',');
                    }
                    sb.push_str(&encoded_attributes);
                    sb.push('}');
                }

                expose.name = sb;

                batch.push(expose);
                Ok(())
            })
        })?;

        self.writer.lock().map_err(From::from).and_then(|mut w| {
            let formatted = match &self.formatter {
                Some(formatter) => formatter.0(batch)?,
                None => format!("{:?}\n", batch),
            };
            w.write_all(formatted.as_bytes())
                .map_err(|e| MetricsError::Other(e.to_string()))
        })
    }
}

/// A formatter for user-defined batch serialization.
struct Formatter(Box<dyn Fn(Vec<ExportLine>) -> Result<String> + Send + Sync>);
impl fmt::Debug for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Formatter(closure)")
    }
}

/// Configuration for a given stdout exporter.
#[derive(Debug)]
pub struct StdoutExporterBuilder<W> {
    writer: Mutex<W>,
    timestamps: bool,
    attribute_encoder: Option<Box<dyn Encoder + Send + Sync>>,
    formatter: Option<Formatter>,
}

impl<W> StdoutExporterBuilder<W>
where
    W: io::Write + fmt::Debug + Send + Sync + 'static,
{
    fn builder() -> StdoutExporterBuilder<io::Stdout> {
        StdoutExporterBuilder {
            writer: Mutex::new(io::stdout()),
            timestamps: true,
            attribute_encoder: None,
            formatter: None,
        }
    }
    /// Set the writer that this exporter will use.
    pub fn with_writer<W2: io::Write>(self, writer: W2) -> StdoutExporterBuilder<W2> {
        StdoutExporterBuilder {
            writer: Mutex::new(writer),
            timestamps: self.timestamps,
            attribute_encoder: self.attribute_encoder,
            formatter: self.formatter,
        }
    }

    /// Hide the timestamps from exported results
    pub fn with_do_not_print_time(self, do_not_print_time: bool) -> Self {
        StdoutExporterBuilder {
            timestamps: do_not_print_time,
            ..self
        }
    }

    /// Set the attribute encoder that this exporter will use.
    pub fn with_attribute_encoder<E>(self, attribute_encoder: E) -> Self
    where
        E: Encoder + Send + Sync + 'static,
    {
        StdoutExporterBuilder {
            attribute_encoder: Some(Box::new(attribute_encoder)),
            ..self
        }
    }

    /// Set a formatter for serializing export batch data
    pub fn with_formatter<T>(self, formatter: T) -> Self
    where
        T: Fn(Vec<ExportLine>) -> Result<String> + Send + Sync + 'static,
    {
        StdoutExporterBuilder {
            formatter: Some(Formatter(Box::new(formatter))),
            ..self
        }
    }

    /// Build a new push controller, returning errors if they arise.
    pub fn build(self) -> Result<StdoutExporter<W>> {
        Ok(StdoutExporter {
            writer: self.writer,
            timestamps: self.timestamps,
            attribute_encoder: self.attribute_encoder.unwrap_or_else(default_encoder),
            formatter: self.formatter,
        })
    }
}
