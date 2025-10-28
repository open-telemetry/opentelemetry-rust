use super::OtlpHttpClient;
use crate::Protocol;
use opentelemetry::{otel_debug, otel_warn};
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    trace::{SpanData, SpanExporter},
};
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

    fn shutdown(&mut self) -> OTelSdkResult {
        let mut client_guard = self.client.lock().map_err(|e| {
            OTelSdkError::InternalFailure(format!("Failed to acquire client lock: {e}"))
        })?;

        if client_guard.take().is_none() {
            return Err(OTelSdkError::AlreadyShutdown);
        }

        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}

/// Handles partial success returned by OTLP endpoints. We log the rejected spans,
/// as well as the error message returned.
fn handle_partial_success(response_body: &[u8], protocol: Protocol) {
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceResponse;

    let response: ExportTraceServiceResponse = match protocol {
        #[cfg(feature = "http-json")]
        Protocol::HttpJson => match serde_json::from_slice(response_body) {
            Ok(r) => r,
            Err(e) => {
                otel_debug!(name: "HttpTraceClient.ResponseParseError", error = e.to_string());
                return;
            }
        },
        _ => match Message::decode(response_body) {
            Ok(r) => r,
            Err(e) => {
                otel_debug!(name: "HttpTraceClient.ResponseParseError", error = e.to_string());
                return;
            }
        },
    };

    if let Some(partial_success) = response.partial_success {
        if partial_success.rejected_spans > 0 || !partial_success.error_message.is_empty() {
            otel_warn!(
                name: "HttpTraceClient.PartialSuccess",
                rejected_spans = partial_success.rejected_spans,
                error_message = partial_success.error_message.as_str(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_invalid_protobuf() {
        // Corrupted/invalid protobuf data
        let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF];

        // Should not panic - logs debug and returns early
        handle_partial_success(&invalid, Protocol::HttpBinary);
    }

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
