//! # Jaeger JSON file Exporter
//!

use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use opentelemetry::sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use opentelemetry::sdk::trace::{TraceRuntime, Tracer};
use opentelemetry::trace::{SpanId, TraceError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// An exporter for jaeger comptible json files containing trace data
#[derive(Debug)]
pub struct JaegerJsonExporter<R> {
    out_path: PathBuf,
    file_prefix: String,
    service_name: String,
    runtime: R,
}

impl<R: JaegerJsonRuntime> JaegerJsonExporter<R> {
    /// Configure a new jaeger-json exporter
    ///
    /// * `out_path` refers to an directory where span data are written. If it does not exist, it is created by the exporter
    /// * `file_prefix` refers to a prefix prependend to each span file
    /// * `service_name` is used to identify the corresponding service in jaeger
    /// * `runtime` specifies the used async runtime to write the trace data
    pub fn new(out_path: PathBuf, file_prefix: String, service_name: String, runtime: R) -> Self {
        Self {
            out_path,
            file_prefix,
            service_name,
            runtime,
        }
    }

    /// Install the exporter using the internal provided runtime
    pub fn install_batch(self) -> Tracer {
        use opentelemetry::trace::TracerProvider;

        let runtime = self.runtime.clone();
        let provider_builder =
            opentelemetry::sdk::trace::TracerProvider::builder().with_batch_exporter(self, runtime);

        let provider = provider_builder.build();

        let tracer =
            provider.versioned_tracer("opentelemetry", Some(env!("CARGO_PKG_VERSION")), None);
        let _ = opentelemetry::global::set_tracer_provider(provider);

        tracer
    }
}

impl<R: JaegerJsonRuntime> SpanExporter for JaegerJsonExporter<R> {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let mut trace_map = HashMap::new();

        for span in batch {
            let ctx = &span.span_context;
            trace_map
                .entry(ctx.trace_id())
                .or_insert_with(Vec::new)
                .push(span_data_to_jaeger_json(span));
        }

        let data = trace_map
            .into_iter()
            .map(|(trace_id, spans)| {
                serde_json::json!({
                    "traceID": trace_id.to_string(),
                    "spans": spans,
                    "processes": {
                        "p1": {
                            "serviceName": self.service_name,
                            "tags": []
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        let json = serde_json::json!({
            "data": data,
        });

        let runtime = self.runtime.clone();
        let out_path = self.out_path.clone();
        let file_prefix = self.file_prefix.clone();

        async move {
            runtime.create_dir(&out_path).await?;

            let file_name = out_path.join(format!(
                "{}-{}.json",
                file_prefix,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("This does not fail")
                    .as_secs()
            ));
            runtime
                .write_to_file(
                    &file_name,
                    &serde_json::to_vec(&json).expect("This is a valid json value"),
                )
                .await?;

            Ok(())
        }
        .boxed()
    }
}

fn span_data_to_jaeger_json(
    span: opentelemetry::sdk::export::trace::SpanData,
) -> serde_json::Value {
    let events = span
        .events
        .iter()
        .map(|e| {
            let mut fields = e
                .attributes
                .iter()
                .map(|a| {
                    let (tpe, value) = opentelemetry_value_to_json(&a.value);
                    serde_json::json!({
                        "key": a.key.as_str(),
                        "type": tpe,
                        "value": value,
                    })
                })
                .collect::<Vec<_>>();
            fields.push(serde_json::json!({
                "key": "event",
                "type": "string",
                "value": e.name,
            }));

            serde_json::json!({
                "timestamp": e.timestamp.duration_since(SystemTime::UNIX_EPOCH).expect("This does not fail").as_micros() as i64,
                "fields": fields,
            })
        })
        .collect::<Vec<_>>();
    let tags = span
        .attributes
        .iter()
        .map(|(key, value)| {
            let (tpe, value) = opentelemetry_value_to_json(value);
            serde_json::json!({
            "key": key.as_str(),
            "type": tpe,
            "value": value,
            })
        })
        .collect::<Vec<_>>();
    let mut references = if span.links.is_empty() {
        None
    } else {
        Some(
            span.links
                .iter()
                .map(|link| {
                    let span_context = &link.span_context;
                    serde_json::json!({
                        "refType": "FOLLOWS_FROM",
                        "traceID": span_context.trace_id().to_string(),
                        "spanID": span_context.span_id().to_string(),
                    })
                })
                .collect::<Vec<_>>(),
        )
    };
    if span.parent_span_id != SpanId::INVALID {
        let val = serde_json::json!({
            "refType": "CHILD_OF",
            "traceID": span.span_context.trace_id().to_string(),
            "spanID": span.parent_span_id.to_string(),
        });
        references.get_or_insert_with(Vec::new).push(val);
    }
    serde_json::json!({
        "traceID": span.span_context.trace_id().to_string(),
        "spanID": span.span_context.span_id().to_string(),
        "startTime": span.start_time.duration_since(SystemTime::UNIX_EPOCH).expect("This does not fail").as_micros() as i64,
        "duration": span.end_time.duration_since(span.start_time).expect("This does not fail").as_micros() as i64,
        "operationName": span.name,
        "tags": tags,
        "logs": events,
        "flags": span.span_context.trace_flags().to_u8(),
        "processID": "p1",
        "warnings": None::<String>,
        "references": references,
    })
}

fn opentelemetry_value_to_json(value: &opentelemetry::Value) -> (&str, serde_json::Value) {
    match value {
        opentelemetry::Value::Bool(b) => ("bool", serde_json::json!(b)),
        opentelemetry::Value::I64(i) => ("int64", serde_json::json!(i)),
        opentelemetry::Value::F64(f) => ("float64", serde_json::json!(f)),
        opentelemetry::Value::String(s) => ("string", serde_json::json!(s.as_str())),
        v @ opentelemetry::Value::Array(_) => ("string", serde_json::json!(v.to_string())),
    }
}

/// Jaeger Json Runtime is an extension to [`TraceRuntime`].
///
/// [`TraceRuntime`]: opentelemetry::sdk::trace::TraceRuntime
#[async_trait]
pub trait JaegerJsonRuntime: TraceRuntime + std::fmt::Debug {
    /// Create a new directory if the given path does not exist yet
    async fn create_dir(&self, path: &Path) -> ExportResult;
    /// Write the provided content to a new file at the given path
    async fn write_to_file(&self, path: &Path, content: &[u8]) -> ExportResult;
}

#[cfg(feature = "rt-tokio")]
#[async_trait]
impl JaegerJsonRuntime for opentelemetry::runtime::Tokio {
    async fn create_dir(&self, path: &Path) -> ExportResult {
        if tokio::fs::metadata(path).await.is_err() {
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| TraceError::Other(Box::new(e)))?
        }

        Ok(())
    }

    async fn write_to_file(&self, path: &Path, content: &[u8]) -> ExportResult {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.write_all(content)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.sync_data()
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;

        Ok(())
    }
}

#[cfg(feature = "rt-tokio-current-thread")]
#[async_trait]
impl JaegerJsonRuntime for opentelemetry::runtime::TokioCurrentThread {
    async fn create_dir(&self, path: &Path) -> ExportResult {
        if tokio::fs::metadata(path).await.is_err() {
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| TraceError::Other(Box::new(e)))?
        }

        Ok(())
    }

    async fn write_to_file(&self, path: &Path, content: &[u8]) -> ExportResult {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.write_all(content)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.sync_data()
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;

        Ok(())
    }
}

#[cfg(feature = "rt-async-std")]
#[async_trait]
impl JaegerJsonRuntime for opentelemetry::runtime::AsyncStd {
    async fn create_dir(&self, path: &Path) -> ExportResult {
        if async_std::fs::metadata(path).await.is_err() {
            async_std::fs::create_dir_all(path)
                .await
                .map_err(|e| TraceError::Other(Box::new(e)))?;
        }
        Ok(())
    }

    async fn write_to_file(&self, path: &Path, content: &[u8]) -> ExportResult {
        use async_std::io::WriteExt;

        let mut file = async_std::fs::File::create(path)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.write_all(content)
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;
        file.sync_data()
            .await
            .map_err(|e| TraceError::Other(Box::new(e)))?;

        Ok(())
    }
}
