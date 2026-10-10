use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use opentelemetry_proto::tonic::trace::v1::TracesData;
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;
use opentelemetry_proto::transform::trace::tonic::group_spans_by_resource_and_scope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry_sdk::Resource;

use crate::output::{JsonLinesWriter, Output};
use crate::ExporterBuildError;

/// Builder for [`SpanExporter`].
#[derive(Debug, Default)]
pub struct SpanExporterBuilder {
    output: Output,
}

impl SpanExporterBuilder {
    /// Write spans to the standard output of the process. This is the default.
    pub fn with_stdout(mut self) -> Self {
        self.output = Output::Stdout;
        self
    }

    /// Append spans to the file at `path`, which is created if it does not exist.
    ///
    /// The file is opened by [`build`](Self::build). The preferred extension is `.jsonl`.
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Output::File(path.into());
        self
    }

    /// Write spans to `writer`.
    ///
    /// Every batch is written as one line with a single `write_all` call, followed by a
    /// `flush`. The writer is dropped when the exporter is shut down.
    pub fn with_writer(mut self, writer: impl Write + Send + 'static) -> Self {
        self.output = Output::Writer(Box::new(writer));
        self
    }

    /// Build the [`SpanExporter`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file configured with [`with_file`](Self::with_file) cannot be
    /// opened for appending.
    pub fn build(self) -> Result<SpanExporter, ExporterBuildError> {
        Ok(SpanExporter::new(self.output.open()?))
    }
}

/// An exporter that writes each batch of spans as one line of OTLP JSON.
///
/// Every line is a `TracesData` object. [`SpanExporter::default`] writes to stdout; use
/// [`SpanExporter::builder`] to write to a file or any other [`Write`] implementation.
pub struct SpanExporter {
    output: JsonLinesWriter,
    resource: ResourceAttributesWithSchema,
}

impl SpanExporter {
    /// Create a builder to configure a [`SpanExporter`].
    pub fn builder() -> SpanExporterBuilder {
        SpanExporterBuilder::default()
    }

    fn new(output: JsonLinesWriter) -> Self {
        SpanExporter {
            output,
            resource: ResourceAttributesWithSchema::default(),
        }
    }
}

impl Default for SpanExporter {
    /// Create an exporter that writes to stdout.
    fn default() -> Self {
        SpanExporter::new(JsonLinesWriter::stdout())
    }
}

impl fmt::Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpanExporter")
            .field("output", &self.output)
            .finish_non_exhaustive()
    }
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        if batch.is_empty() {
            return self.output.ensure_open();
        }
        let resource_spans = group_spans_by_resource_and_scope(batch, &self.resource);
        self.output.write(&TracesData { resource_spans })
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.output.shutdown()
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.output.flush()
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
    use opentelemetry::trace::{
        SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState, Tracer,
        TracerProvider as _,
    };
    use opentelemetry::{InstrumentationScope, KeyValue};
    use opentelemetry_sdk::error::OTelSdkError;
    use opentelemetry_sdk::trace::{SdkTracerProvider, SpanEvents, SpanExporter as _, SpanLinks};
    use serde_json::{json, Value};
    use std::borrow::Cow;
    use std::time::{Duration, SystemTime};

    fn span(name: &'static str, scope: &'static str) -> SpanData {
        SpanData {
            span_context: SpanContext::new(
                TraceId::from(0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10),
                SpanId::from(0x1112_1314_1516_1718),
                TraceFlags::SAMPLED,
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Server,
            name: Cow::Borrowed(name),
            // Multiples of 100 ns, the resolution of `SystemTime` on Windows.
            start_time: SystemTime::UNIX_EPOCH + Duration::from_nanos(1_581_452_772_000_000_300),
            end_time: SystemTime::UNIX_EPOCH + Duration::from_nanos(1_581_452_773_000_000_700),
            attributes: vec![KeyValue::new("http.response.status_code", 200)],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Ok,
            instrumentation_scope: InstrumentationScope::builder(scope).build(),
        }
    }

    fn resource() -> Resource {
        Resource::builder_empty()
            .with_service_name("checkout")
            .build()
    }

    fn exporter(buffer: &SharedBuffer) -> SpanExporter {
        let mut exporter = SpanExporter::builder()
            .with_writer(buffer.clone())
            .build()
            .unwrap();
        exporter.set_resource(&resource());
        exporter
    }

    #[test]
    fn writes_a_batch_as_one_traces_data_line() {
        let buffer = SharedBuffer::default();
        let exporter = exporter(&buffer);
        let batch = vec![span("GET /cart", "scope-a"), span("SELECT cart", "scope-b")];

        block_on(exporter.export(batch.clone())).unwrap();

        let lines = buffer.lines();
        assert_eq!(lines.len(), 1);
        let decoded: TracesData = serde_json::from_str(&lines[0]).unwrap();
        let expected = TracesData {
            resource_spans: group_spans_by_resource_and_scope(batch, &(&resource()).into()),
        };
        assert_eq!(decoded, expected);
    }

    #[test]
    fn uses_the_otlp_json_encoding() {
        let buffer = SharedBuffer::default();
        let exporter = exporter(&buffer);

        block_on(exporter.export(vec![span("GET /cart", "scope-a")])).unwrap();

        let line: Value = serde_json::from_str(&buffer.lines()[0]).unwrap();
        let resource_spans = &line["resourceSpans"][0];
        assert_eq!(
            resource_spans["resource"]["attributes"][0],
            json!({"key": "service.name", "value": {"stringValue": "checkout"}})
        );
        assert_eq!(resource_spans["scopeSpans"][0]["scope"]["name"], "scope-a");
        let span = &resource_spans["scopeSpans"][0]["spans"][0];
        assert_eq!(span["traceId"], "0102030405060708090a0b0c0d0e0f10");
        assert_eq!(span["spanId"], "1112131415161718");
        assert_eq!(span["parentSpanId"], "");
        assert_eq!(span["name"], "GET /cart");
        assert_eq!(span["kind"], 2);
        assert_eq!(span["startTimeUnixNano"], "1581452772000000300");
        assert_eq!(span["endTimeUnixNano"], "1581452773000000700");
        assert_eq!(
            span["attributes"][0],
            json!({"key": "http.response.status_code", "value": {"intValue": "200"}})
        );
        assert_eq!(span["status"]["code"], 1);
    }

    #[test]
    fn empty_batches_are_not_written() {
        let buffer = SharedBuffer::default();
        let exporter = exporter(&buffer);

        block_on(exporter.export(Vec::new())).unwrap();

        assert_eq!(buffer.contents(), "");
    }

    #[test]
    fn fails_after_shutdown() {
        let buffer = SharedBuffer::default();
        let exporter = exporter(&buffer);

        exporter.shutdown().unwrap();

        assert!(matches!(
            block_on(exporter.export(vec![span("GET /cart", "scope-a")])),
            Err(OTelSdkError::AlreadyShutdown)
        ));
        assert!(matches!(
            block_on(exporter.export(Vec::new())),
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
        assert_eq!(buffer.contents(), "");
    }

    #[test]
    fn appends_one_line_per_batch_to_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.jsonl");
        let exporter = SpanExporter::builder().with_file(&path).build().unwrap();

        block_on(exporter.export(vec![span("first", "scope-a")])).unwrap();
        block_on(exporter.export(vec![span("second", "scope-a")])).unwrap();
        exporter.shutdown().unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let names: Vec<String> = contents
            .lines()
            .map(|line| {
                let data: TracesData = serde_json::from_str(line).unwrap();
                data.resource_spans[0].scope_spans[0].spans[0].name.clone()
            })
            .collect();
        assert_eq!(names, ["first", "second"]);
    }

    #[test]
    fn exports_spans_from_a_tracer_provider() {
        let buffer = SharedBuffer::default();
        let exporter = SpanExporter::builder()
            .with_writer(buffer.clone())
            .build()
            .unwrap();
        let provider = SdkTracerProvider::builder()
            .with_resource(resource())
            .with_simple_exporter(exporter)
            .build();

        provider.tracer("checkout").in_span("parent", |_| {
            provider.tracer("checkout").in_span("child", |_| {});
        });
        provider.shutdown().unwrap();

        // The simple processor exports each span as soon as it ends.
        let spans: Vec<_> = buffer
            .lines()
            .iter()
            .map(|line| {
                let mut data: TracesData = serde_json::from_str(line).unwrap();
                let mut resource_spans = data.resource_spans.remove(0);
                let service = &resource_spans.resource.as_ref().unwrap().attributes[0];
                assert_eq!(service.key, "service.name");
                resource_spans.scope_spans.remove(0).spans.remove(0)
            })
            .collect();
        let [child, parent] = &spans[..] else {
            panic!("expected two lines, got {spans:?}");
        };
        assert_eq!(
            (child.name.as_str(), parent.name.as_str()),
            ("child", "parent")
        );
        assert_eq!(child.parent_span_id, parent.span_id);
        assert_eq!(child.trace_id, parent.trace_id);
        assert!(parent.parent_span_id.is_empty());
    }

    #[test]
    fn default_writes_to_stdout() {
        assert_eq!(
            format!("{:?}", SpanExporter::default()),
            "SpanExporter { output: JsonLinesWriter { destination: Stdout, .. }, .. }"
        );
    }
}
