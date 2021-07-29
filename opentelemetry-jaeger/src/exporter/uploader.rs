//! # Jaeger Span Uploader
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
use crate::exporter::collector;
use crate::exporter::{agent, jaeger};
use async_trait::async_trait;
use opentelemetry::sdk::export::trace;

use crate::exporter::JaegerTraceRuntime;

#[async_trait]
pub(crate) trait Uploader: std::fmt::Debug + Send {
    async fn upload(&mut self, batch: jaeger::Batch) -> trace::ExportResult;
}

#[derive(Debug)]
pub(crate) enum SyncUploader {
    Agent(agent::AgentSyncClientUdp),
}

#[async_trait]
impl Uploader for SyncUploader {
    async fn upload(&mut self, batch: jaeger::Batch) -> trace::ExportResult {
        match self {
            SyncUploader::Agent(client) => {
                // TODO Implement retry behaviour
                client
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
    Agent(agent::AgentAsyncClientUdp<R>),
    /// Collector sync client
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    Collector(collector::CollectorAsyncClientHttp),
}

#[async_trait]
impl<R: JaegerTraceRuntime> Uploader for AsyncUploader<R> {
    /// Emit a jaeger batch for the given uploader
    async fn upload(&mut self, batch: jaeger::Batch) -> trace::ExportResult {
        match self {
            AsyncUploader::Agent(client) => {
                // TODO Implement retry behaviour
                client
                    .emit_batch(batch)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;
            }
            #[cfg(feature = "collector_client")]
            AsyncUploader::Collector(collector) => {
                // TODO Implement retry behaviour
                collector.submit_batch(batch).await?;
            }
            #[cfg(all(not(feature = "collector_client"), feature = "wasm_collector_client"))]
            AsyncUploader::Collector(collector) => {
                // TODO Implement retry behaviour
                collector
                    .submit_batch(batch)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;
            }
        }
        Ok(())
    }
}
