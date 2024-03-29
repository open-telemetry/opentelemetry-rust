use std::sync::Arc;

use futures_core::future::BoxFuture;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};

use super::OtlpHttpClient;

impl SpanExporter for OtlpHttpClient {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let client = match self
            .client
            .lock()
            .map_err(|e| TraceError::Other(e.to_string().into()))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(TraceError::Other("exporter is already shut down".into())),
            }) {
            Ok(client) => client,
            Err(err) => return Box::pin(std::future::ready(Err(err))),
        };

        let (body, content_type) = match build_body(batch) {
            Ok(body) => body,
            Err(e) => return Box::pin(std::future::ready(Err(e))),
        };

        let mut request = match http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type)
            .body(body)
        {
            Ok(req) => req,
            Err(e) => {
                return Box::pin(std::future::ready(Err(crate::Error::RequestFailed(
                    Box::new(e),
                )
                .into())))
            }
        };

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        Box::pin(async move {
            let request_uri = request.uri().to_string();
            let response = client.send(request).await?;

            if !response.status().is_success() {
                let error = format!(
                    "OpenTelemetry trace export failed. Url: {}, Status Code: {}, Response: {:?}",
                    response.status().as_u16(),
                    request_uri,
                    response.body()
                );
                return Err(TraceError::Other(error.into()));
            }

            Ok(())
        })
    }

    fn shutdown(&mut self) {
        let _ = self.client.lock().map(|mut c| c.take());
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
    use crate::exporter::default_protocol;
    #[cfg(feature = "http-json")]
    use crate::Protocol;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use prost::Message;

    let req = ExportTraceServiceRequest {
        resource_spans: spans.into_iter().map(Into::into).collect(),
    };
    match default_protocol() {
        #[cfg(feature = "http-json")]
        Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
            Ok(json) => Ok((json.into(), "application/json")),
            Err(e) => Err(TraceError::from(e.to_string())),
        },
        _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
    }
}

#[cfg(not(any(feature = "http-proto", feature = "http-json")))]
fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
    Err(TraceError::Other(
        "No http protocol configured. Enable one via `http-proto` or `http-json`".into(),
    ))
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
mod tests {
    #[test]
    fn test_build_body() {
        use crate::exporter::http::trace::build_body;
        use opentelemetry::trace::{
            SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
        };
        use opentelemetry_sdk::export::trace::SpanData;
        use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};
        use opentelemetry_sdk::Resource;
        use std::{borrow::Cow, time::SystemTime};

        let span_data = (0..5)
            .map(|_| SpanData {
                span_context: SpanContext::new(
                    TraceId::from_u128(12),
                    SpanId::from_u64(12),
                    TraceFlags::default(),
                    false,
                    TraceState::default(),
                ),
                parent_span_id: SpanId::from_u64(12),
                span_kind: SpanKind::Client,
                name: Default::default(),
                start_time: SystemTime::now(),
                end_time: SystemTime::now(),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                events: SpanEvents::default(),
                links: SpanLinks::default(),
                status: Status::Unset,
                resource: Cow::Owned(Resource::empty()),
                instrumentation_lib: Default::default(),
            })
            .collect::<Vec<SpanData>>();

        let result = build_body(span_data).unwrap();
        match crate::exporter::default_protocol() {
            #[cfg(feature = "http-json")]
            crate::Protocol::HttpJson => {
                assert_eq!(result.1, "application/json")
            }
            _ => {
                assert_eq!(result.1, "application/x-protobuf")
            }
        };
    }
}
