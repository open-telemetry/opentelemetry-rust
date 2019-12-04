//! # Jaeger Span Uploader
use crate::{agent, jaeger, thrift::agent::TAgentSyncClient};
#[cfg(feature = "collector_client")]
use crate::{collector, thrift::jaeger::TCollectorSyncClient};
use opentelemetry::exporter::trace;

/// Uploads a batch of spans to Jaeger
#[derive(Debug)]
pub(crate) enum BatchUploader {
    /// Agent sync client
    Agent(agent::AgentSyncClientUDP),
    /// Collector sync client
    #[cfg(feature = "collector_client")]
    Collector(collector::CollectorSyncClientHttp),
}

impl BatchUploader {
    /// Emit a jaeger batch for the given uploader
    pub(crate) fn upload(&mut self, batch: jaeger::Batch) -> trace::ExportResult {
        match self {
            BatchUploader::Agent(client) => match client.emit_batch(batch) {
                Ok(_) => trace::ExportResult::Success,
                // TODO determine if the error is retryable
                Err(_) => trace::ExportResult::FailedNotRetryable,
            },
            #[cfg(feature = "collector_client")]
            BatchUploader::Collector(collector) => match collector.submit_batches(vec![batch]) {
                Ok(_) => trace::ExportResult::Success,
                // TODO determine if the error is retryable
                Err(_) => trace::ExportResult::FailedNotRetryable,
            },
        }
    }
}
