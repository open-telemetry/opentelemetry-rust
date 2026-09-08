use super::OtlpHttpClient;
use crate::Protocol;
use opentelemetry::{otel_debug, otel_warn};
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    trace::{SpanData, SpanExporter},
};
#[cfg(feature = "http-proto")]
use prost::Message;

impl SpanExporter for OtlpHttpClient {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let response_body = self
            .export_http_with_retry(
                batch,
                OtlpHttpClient::build_trace_export_body,
                "HttpTracesClient.Export",
            )
            .await?;

        handle_partial_success(&response_body, self.protocol);
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        let mut client_guard = self.client.lock().map_err(|e| {
            OTelSdkError::InternalFailure(format!("Failed to acquire client lock: {e}"))
        })?;

        if client_guard.take().is_none() {
            return Err(OTelSdkError::AlreadyShutdown);
        }

        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.clone();
    }
}

/// Handles partial success returned by OTLP endpoints. We log the rejected spans,
/// as well as the error message returned.
fn handle_partial_success(response_body: &[u8], protocol: Protocol) {
    let partial_success = match protocol {
        #[cfg(feature = "http-json")]
        Protocol::HttpJson => {
            use opentelemetry_proto::json::collector::trace::v1::ExportTraceServiceResponse;
            match serde_json::from_slice::<ExportTraceServiceResponse>(response_body) {
                Ok(r) => r
                    .partial_success
                    .map(|p| (p.rejected_spans, p.error_message)),
                Err(e) => {
                    otel_debug!(name: "HttpTraceClient.ResponseParseError", error = e.to_string());
                    return;
                }
            }
        }
        #[cfg(feature = "http-proto")]
        Protocol::HttpBinary => {
            use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceResponse;
            match ExportTraceServiceResponse::decode(response_body) {
                Ok(r) => r
                    .partial_success
                    .map(|p| (p.rejected_spans, p.error_message)),
                Err(e) => {
                    otel_debug!(name: "HttpTraceClient.ResponseParseError", error = e.to_string());
                    return;
                }
            }
        }
        #[cfg(feature = "grpc-tonic")]
        Protocol::Grpc => {
            unreachable!("HTTP client should not receive Grpc protocol")
        }
    };

    if let Some((rejected_spans, error_message)) = partial_success {
        if rejected_spans > 0 || !error_message.is_empty() {
            otel_warn!(
                name: "HttpTraceClient.PartialSuccess",
                rejected_spans = rejected_spans,
                error_message = error_message.as_str(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "http-proto")]
    #[test]
    fn test_handle_invalid_protobuf() {
        // Corrupted/invalid protobuf data
        let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF];

        // Should not panic - logs debug and returns early
        handle_partial_success(&invalid, Protocol::HttpBinary);
    }

    #[cfg(feature = "http-proto")]
    #[test]
    fn test_handle_empty_response() {
        let empty = vec![];

        // Should not panic
        handle_partial_success(&empty, Protocol::HttpBinary);
    }

    #[cfg(feature = "http-json")]
    #[test]
    fn test_handle_invalid_json() {
        let invalid_json = b"{not valid json}";

        // Should not panic - logs debug and returns
        handle_partial_success(invalid_json, Protocol::HttpJson);
    }
}
