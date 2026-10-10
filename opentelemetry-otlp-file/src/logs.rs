use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use opentelemetry_proto::tonic::logs::v1::LogsData;
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;
use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::LogBatch;
use opentelemetry_sdk::Resource;

use crate::output::{JsonLinesWriter, Output};
use crate::ExporterBuildError;

/// Builder for [`LogExporter`].
#[derive(Debug, Default)]
pub struct LogExporterBuilder {
    output: Output,
}

impl LogExporterBuilder {
    /// Write log records to the standard output of the process. This is the default.
    pub fn with_stdout(mut self) -> Self {
        self.output = Output::Stdout;
        self
    }

    /// Append log records to the file at `path`, which is created if it does not exist.
    ///
    /// The file is opened by [`build`](Self::build). The preferred extension is `.jsonl`.
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Output::File(path.into());
        self
    }

    /// Write log records to `writer`.
    ///
    /// Every batch is written as one line with a single `write_all` call, followed by a
    /// `flush`. The writer is dropped when the exporter is shut down.
    pub fn with_writer(mut self, writer: impl Write + Send + 'static) -> Self {
        self.output = Output::Writer(Box::new(writer));
        self
    }

    /// Build the [`LogExporter`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file configured with [`with_file`](Self::with_file) cannot be
    /// opened for appending.
    pub fn build(self) -> Result<LogExporter, ExporterBuildError> {
        Ok(LogExporter::new(self.output.open()?))
    }
}

/// An exporter that writes each batch of log records as one line of OTLP JSON.
///
/// Every line is a `LogsData` object. [`LogExporter::default`] writes to stdout; use
/// [`LogExporter::builder`] to write to a file or any other [`Write`] implementation.
pub struct LogExporter {
    output: JsonLinesWriter,
    resource: ResourceAttributesWithSchema,
}

impl LogExporter {
    /// Create a builder to configure a [`LogExporter`].
    pub fn builder() -> LogExporterBuilder {
        LogExporterBuilder::default()
    }

    fn new(output: JsonLinesWriter) -> Self {
        LogExporter {
            output,
            resource: ResourceAttributesWithSchema::default(),
        }
    }
}

impl Default for LogExporter {
    /// Create an exporter that writes to stdout.
    fn default() -> Self {
        LogExporter::new(JsonLinesWriter::stdout())
    }
}

impl fmt::Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogExporter")
            .field("output", &self.output)
            .finish_non_exhaustive()
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        if batch.iter().next().is_none() {
            return self.output.ensure_open();
        }
        let resource_logs = group_logs_by_resource_and_scope(&batch, &self.resource);
        self.output.write(&LogsData { resource_logs })
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.output.shutdown()
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::tests::SharedBuffer;
    use futures_executor::block_on;
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use opentelemetry_sdk::error::OTelSdkError;
    use opentelemetry_sdk::logs::{LogExporter as _, SdkLoggerProvider};
    use serde_json::{json, Value};
    use tracing_subscriber::layer::SubscriberExt;

    /// Emits log records through the `tracing` appender and returns the exported lines.
    fn export_logs(emit: impl FnOnce()) -> Vec<String> {
        let buffer = SharedBuffer::default();
        let exporter = LogExporter::builder()
            .with_writer(buffer.clone())
            .build()
            .unwrap();
        let provider = SdkLoggerProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_service_name("checkout")
                    .build(),
            )
            .with_simple_exporter(exporter)
            .build();
        let subscriber =
            tracing_subscriber::registry().with(OpenTelemetryTracingBridge::new(&provider));

        tracing::subscriber::with_default(subscriber, emit);
        provider.shutdown().unwrap();

        buffer.lines()
    }

    #[test]
    fn writes_a_batch_as_one_logs_data_line() {
        let lines = export_logs(|| {
            tracing::info!(name: "order.placed", target: "checkout", order_id = 42, "order placed");
        });

        assert_eq!(lines.len(), 1);
        let decoded: LogsData = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(decoded.resource_logs[0].scope_logs[0].log_records.len(), 1);

        let line: Value = serde_json::from_str(&lines[0]).unwrap();
        let resource_logs = &line["resourceLogs"][0];
        assert_eq!(
            resource_logs["resource"]["attributes"][0],
            json!({"key": "service.name", "value": {"stringValue": "checkout"}})
        );
        assert_eq!(resource_logs["scopeLogs"][0]["scope"]["name"], "checkout");
        let record = &resource_logs["scopeLogs"][0]["logRecords"][0];
        assert_eq!(record["severityNumber"], 9);
        assert_eq!(record["severityText"], "INFO");
        assert_eq!(record["eventName"], "order.placed");
        assert_eq!(record["body"], json!({"stringValue": "order placed"}));
        // Look the attribute up by key: optional appender features add more attributes.
        let order_id = record["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|attribute| attribute["key"] == "order_id");
        assert_eq!(
            order_id,
            Some(&json!({"key": "order_id", "value": {"intValue": "42"}}))
        );
        assert!(record["observedTimeUnixNano"].is_string());
    }

    #[cfg(feature = "trace")]
    #[test]
    fn includes_the_active_trace_context() {
        use opentelemetry::trace::{Tracer, TracerProvider as _};
        use opentelemetry_sdk::trace::SdkTracerProvider;

        let tracer = SdkTracerProvider::builder().build().tracer("checkout");
        let mut context = None;
        let lines = export_logs(|| {
            tracer.in_span("checkout", |cx| {
                use opentelemetry::trace::TraceContextExt;
                context = Some(cx.span().span_context().clone());
                tracing::warn!("payment retried");
            });
        });

        let context = context.unwrap();
        let line: Value = serde_json::from_str(&lines[0]).unwrap();
        let record = &line["resourceLogs"][0]["scopeLogs"][0]["logRecords"][0];
        assert_eq!(record["traceId"], context.trace_id().to_string());
        assert_eq!(record["spanId"], context.span_id().to_string());
        assert_eq!(record["severityNumber"], 13);
    }

    #[test]
    fn empty_batches_are_not_written() {
        let buffer = SharedBuffer::default();
        let exporter = LogExporter::builder()
            .with_writer(buffer.clone())
            .build()
            .unwrap();

        block_on(exporter.export(LogBatch::new(&[]))).unwrap();

        assert_eq!(buffer.contents(), "");
    }

    #[test]
    fn fails_after_shutdown() {
        let exporter = LogExporter::builder()
            .with_writer(SharedBuffer::default())
            .build()
            .unwrap();

        exporter.shutdown().unwrap();

        assert!(matches!(
            block_on(exporter.export(LogBatch::new(&[]))),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(matches!(
            exporter.shutdown(),
            Err(OTelSdkError::AlreadyShutdown)
        ));
    }
}
