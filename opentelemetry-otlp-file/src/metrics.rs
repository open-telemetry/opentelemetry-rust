use std::env::{self, VarError};
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use opentelemetry::otel_warn;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::metrics::v1::MetricsData;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use opentelemetry_sdk::metrics::Temporality;

use crate::output::{JsonLinesWriter, Output};
use crate::ExporterBuildError;

const OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: &str =
    "OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE";

/// Builder for [`MetricExporter`].
#[derive(Debug, Default)]
pub struct MetricExporterBuilder {
    output: Output,
    temporality: Option<Temporality>,
}

impl MetricExporterBuilder {
    /// Write metrics to the standard output of the process. This is the default.
    pub fn with_stdout(mut self) -> Self {
        self.output = Output::Stdout;
        self
    }

    /// Append metrics to the file at `path`, which is created if it does not exist.
    ///
    /// The file is opened by [`build`](Self::build). The preferred extension is `.jsonl`.
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Output::File(path.into());
        self
    }

    /// Write metrics to `writer`.
    ///
    /// Every export is written as one line with a single `write_all` call, followed by a
    /// `flush`. The writer is dropped when the exporter is shut down.
    pub fn with_writer(mut self, writer: impl Write + Send + 'static) -> Self {
        self.output = Output::Writer(Box::new(writer));
        self
    }

    /// Set the temporality of exported metrics.
    ///
    /// This takes precedence over the `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`
    /// environment variable. The default is [`Temporality::Cumulative`].
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Build the [`MetricExporter`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file configured with [`with_file`](Self::with_file) cannot be
    /// opened for appending.
    pub fn build(self) -> Result<MetricExporter, ExporterBuildError> {
        Ok(MetricExporter {
            output: self.output.open()?,
            temporality: resolve_temporality(self.temporality),
        })
    }
}

/// Resolve the temporality from, in order of precedence, the builder, the
/// `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` environment variable and the default.
fn resolve_temporality(configured: Option<Temporality>) -> Temporality {
    if let Some(temporality) = configured {
        return temporality;
    }
    match env::var(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE) {
        Ok(value) if !value.is_empty() => value.parse().unwrap_or_else(|()| {
            warn_invalid_temporality(&value, "expected 'cumulative', 'delta', or 'lowmemory'");
            Temporality::default()
        }),
        Ok(_) | Err(VarError::NotPresent) => Temporality::default(),
        Err(VarError::NotUnicode(_)) => {
            warn_invalid_temporality("<non-Unicode>", "value is not valid Unicode");
            Temporality::default()
        }
    }
}

fn warn_invalid_temporality(value: &str, reason: &str) {
    let message = format!(
        "Ignoring value '{value}' for {OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE}: {reason}"
    );
    otel_warn!(
        name: "Exporter.Config.InvalidEnvironmentVariable",
        message = message.as_str(),
        environment_variable = OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE,
        value = value,
        reason = reason
    );
}

/// An exporter that writes each collection of metrics as one line of OTLP JSON.
///
/// Every line is a `MetricsData` object. [`MetricExporter::default`] writes to stdout; use
/// [`MetricExporter::builder`] to write to a file or any other [`Write`] implementation.
pub struct MetricExporter {
    output: JsonLinesWriter,
    temporality: Temporality,
}

impl MetricExporter {
    /// Create a builder to configure a [`MetricExporter`].
    pub fn builder() -> MetricExporterBuilder {
        MetricExporterBuilder::default()
    }
}

impl Default for MetricExporter {
    /// Create an exporter that writes to stdout.
    ///
    /// The temporality is read from `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`,
    /// defaulting to [`Temporality::Cumulative`].
    fn default() -> Self {
        MetricExporter {
            output: JsonLinesWriter::stdout(),
            temporality: resolve_temporality(None),
        }
    }
}

impl fmt::Debug for MetricExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetricExporter")
            .field("output", &self.output)
            .field("temporality", &self.temporality)
            .finish()
    }
}

impl PushMetricExporter for MetricExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        if metrics.scope_metrics().next().is_none() {
            return self.output.ensure_open();
        }
        let ExportMetricsServiceRequest { resource_metrics } = metrics.into();
        self.output.write(&MetricsData { resource_metrics })
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.output.flush()
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.output.shutdown()
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::tests::SharedBuffer;
    use futures_executor::block_on;
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry::KeyValue;
    use opentelemetry_proto::tonic::metrics::v1::{metric, number_data_point};
    use opentelemetry_sdk::error::OTelSdkError;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use opentelemetry_sdk::Resource;
    use serde_json::{json, Value};

    fn resolve_with_env(value: Option<&str>, configured: Option<Temporality>) -> Temporality {
        temp_env::with_var(
            OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE,
            value,
            || resolve_temporality(configured),
        )
    }

    #[test]
    fn temporality_defaults_to_cumulative() {
        assert_eq!(resolve_with_env(None, None), Temporality::Cumulative);
        assert_eq!(resolve_with_env(Some(""), None), Temporality::Cumulative);
    }

    #[test]
    fn temporality_is_read_from_the_environment() {
        assert_eq!(resolve_with_env(Some("delta"), None), Temporality::Delta);
        assert_eq!(resolve_with_env(Some("Delta"), None), Temporality::Delta);
        assert_eq!(
            resolve_with_env(Some("LowMemory"), None),
            Temporality::LowMemory
        );
        assert_eq!(
            resolve_with_env(Some("cumulative"), None),
            Temporality::Cumulative
        );
    }

    #[test]
    fn invalid_temporality_in_the_environment_is_ignored() {
        assert_eq!(
            resolve_with_env(Some("sometimes"), None),
            Temporality::Cumulative
        );
    }

    #[test]
    fn configured_temporality_overrides_the_environment() {
        assert_eq!(
            resolve_with_env(Some("delta"), Some(Temporality::Cumulative)),
            Temporality::Cumulative
        );
        temp_env::with_var(
            OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE,
            Some("cumulative"),
            || {
                let exporter = MetricExporter::builder()
                    .with_writer(SharedBuffer::default())
                    .with_temporality(Temporality::Delta)
                    .build()
                    .unwrap();
                assert_eq!(exporter.temporality(), Temporality::Delta);
            },
        );
    }

    /// Records a counter and a histogram, then returns the first exported line.
    fn export_once(temporality: Temporality) -> String {
        let buffer = SharedBuffer::default();
        let exporter = MetricExporter::builder()
            .with_writer(buffer.clone())
            .with_temporality(temporality)
            .build()
            .unwrap();
        let provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_service_name("checkout")
                    .build(),
            )
            .with_periodic_exporter(exporter)
            .build();
        let meter = provider.meter("checkout");
        meter
            .u64_counter("orders")
            .build()
            .add(3, &[KeyValue::new("payment.method", "card")]);
        meter.f64_histogram("order.value").build().record(12.5, &[]);

        provider.force_flush().unwrap();
        provider.shutdown().unwrap();

        buffer.lines().into_iter().next().unwrap()
    }

    fn metric<'a>(line: &'a Value, name: &str) -> &'a Value {
        line["resourceMetrics"][0]["scopeMetrics"][0]["metrics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|metric| metric["name"] == name)
            .unwrap()
    }

    #[test]
    fn writes_a_collection_as_one_metrics_data_line() {
        let line = export_once(Temporality::Cumulative);

        let decoded: MetricsData = serde_json::from_str(&line).unwrap();
        assert_eq!(decoded.resource_metrics.len(), 1);
        let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metrics.len(), 2);
        let orders = metrics
            .iter()
            .find(|metric| metric.name == "orders")
            .unwrap();
        match &orders.data {
            Some(metric::Data::Sum(sum)) => assert_eq!(
                sum.data_points[0].value,
                Some(number_data_point::Value::AsInt(3))
            ),
            other => panic!("unexpected data: {other:?}"),
        }

        let line: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            line["resourceMetrics"][0]["resource"]["attributes"][0],
            json!({"key": "service.name", "value": {"stringValue": "checkout"}})
        );
        let orders = &metric(&line, "orders")["sum"];
        assert_eq!(orders["aggregationTemporality"], 2);
        assert_eq!(orders["isMonotonic"], true);
        assert_eq!(
            orders["dataPoints"][0]["attributes"][0],
            json!({"key": "payment.method", "value": {"stringValue": "card"}})
        );
        let order_value = &metric(&line, "order.value")["histogram"]["dataPoints"][0];
        assert_eq!(order_value["count"], "1");
        assert_eq!(order_value["sum"], 12.5);
        assert!(order_value["bucketCounts"]
            .as_array()
            .unwrap()
            .iter()
            .all(Value::is_string));
    }

    #[test]
    fn writes_the_configured_temporality() {
        let line: Value = serde_json::from_str(&export_once(Temporality::Delta)).unwrap();

        assert_eq!(metric(&line, "orders")["sum"]["aggregationTemporality"], 1);
        assert_eq!(
            metric(&line, "order.value")["histogram"]["aggregationTemporality"],
            1
        );
    }

    #[test]
    fn empty_collections_are_not_written() {
        let buffer = SharedBuffer::default();
        let exporter = MetricExporter::builder()
            .with_writer(buffer.clone())
            .build()
            .unwrap();

        block_on(exporter.export(&ResourceMetrics::default())).unwrap();

        assert_eq!(buffer.contents(), "");
    }

    #[test]
    fn fails_after_shutdown() {
        let exporter = MetricExporter::builder()
            .with_writer(SharedBuffer::default())
            .build()
            .unwrap();

        exporter.shutdown().unwrap();

        assert!(matches!(
            block_on(exporter.export(&ResourceMetrics::default())),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(matches!(
            exporter.force_flush(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(matches!(
            exporter.shutdown(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
    }
}
