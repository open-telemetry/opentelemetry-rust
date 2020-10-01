//! # Jaeger Span Uploader
#[cfg(feature = "collector_client")]
use crate::collector;
use crate::{agent, jaeger};
use opentelemetry::exporter::trace;

/// Uploads a batch of spans to Jaeger
#[derive(Debug)]
pub(crate) enum BatchUploader {
    /// Agent sync client
    Agent(agent::AgentAsyncClientUDP),
    /// Collector sync client
    #[cfg(feature = "collector_client")]
    Collector(collector::CollectorSyncClientHttp),
}

impl BatchUploader {
    /// Emit a jaeger batch for the given uploader
    pub(crate) async fn upload(&self, batch: jaeger::Batch) -> trace::ExportResult {
        match self {
            BatchUploader::Agent(client) => match client.emit_batch(batch).await {
                Ok(_) => trace::ExportResult::Success,
                // TODO determine if the error is retryable
                Err(_) => trace::ExportResult::FailedNotRetryable,
            },
            #[cfg(feature = "collector_client")]
            BatchUploader::Collector(collector) => {
                match collector.submit_batch(batch).await {
                    Ok(_) => trace::ExportResult::Success,
                    // TODO determine if the error is retryable
                    Err(_) => trace::ExportResult::FailedNotRetryable,
                }
            }
        }
    }
}
