use std::sync::Arc;

use futures_core::future::BoxFuture;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::trace::{TraceError, TraceResult};
use opentelemetry_http::ResponseExt;
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
                eprintln!(
                    "OpenTelemetry export failed. Url: {}, Response: {:?}",
                    request_uri, response
                );
            }

            Ok(())
        })
    }

    fn shutdown(&mut self) {
        let _ = self.client.lock().map(|mut c| c.take());
    }
}

#[cfg(feature = "http-proto")]
fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use prost::Message;

    let req = ExportTraceServiceRequest {
        resource_spans: spans.into_iter().map(Into::into).collect(),
    };
    let mut buf = vec![];
    req.encode(&mut buf).map_err(crate::Error::from)?;

    Ok((buf, "application/x-protobuf"))
}

#[cfg(not(feature = "http-proto"))]
fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
    Err(TraceError::Other(
        "No http protocol configured. Enable one via `http-proto`".into(),
    ))
}
