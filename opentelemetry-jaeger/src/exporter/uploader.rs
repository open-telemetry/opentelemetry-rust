//! # Jaeger Span Uploader
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
use crate::exporter::collector;
use crate::exporter::{agent, jaeger};
use async_trait::async_trait;
use opentelemetry_sdk::export::trace::ExportResult;
use std::fmt::Debug;

use crate::exporter::thrift::jaeger::Batch;
use crate::exporter::JaegerTraceRuntime;

#[async_trait]
pub(crate) trait Uploader: Debug + Send + Sync {
    async fn upload(&self, batch: jaeger::Batch) -> ExportResult;
}

#[derive(Debug)]
pub(crate) enum SyncUploader {
    Agent(std::sync::Mutex<agent::AgentSyncClientUdp>),
}

#[async_trait]
impl Uploader for SyncUploader {
    async fn upload(&self, batch: jaeger::Batch) -> ExportResult {
        match self {
            SyncUploader::Agent(client) => {
                // TODO Implement retry behavior
                client
                    .lock()
                    .expect("Failed to lock agent client")
                    .emit_batch(batch)
                    .map_err::<crate::Error, _>(Into::into)?;
            }
        }
        Ok(())
    }
}

/// Uploads a batch of spans to Jaeger
#[derive(Debug)]
pub(crate) enum AsyncUploader<R: JaegerTraceRuntime> {
    /// Agent async client
    Agent(futures_util::lock::Mutex<agent::AgentAsyncClientUdp<R>>),
    /// Collector sync client
    #[cfg(feature = "collector_client")]
    Collector(collector::AsyncHttpClient),
    #[cfg(feature = "wasm_collector_client")]
    WasmCollector(collector::WasmCollector),
}

#[async_trait]
impl<R: JaegerTraceRuntime> Uploader for AsyncUploader<R> {
    async fn upload(&self, batch: Batch) -> ExportResult {
        match self {
            Self::Agent(client) => {
                // TODO Implement retry behaviour
                client
                    .lock()
                    .await
                    .emit_batch(batch)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;
            }
            #[cfg(feature = "collector_client")]
            Self::Collector(collector) => {
                // TODO Implement retry behaviour
                collector.submit_batch(batch).await?;
            }
            #[cfg(feature = "wasm_collector_client")]
            Self::WasmCollector(collector) => {
                collector
                    .submit_batch(batch)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;
            }
        }
        Ok(())
    }
}
