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
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use prost::Message;

    use crate::{exporter::default_protocol, Protocol};

    let req = ExportTraceServiceRequest {
        resource_spans: spans.into_iter().map(Into::into).collect(),
    };
    let buf;
    let ctype;
    match default_protocol() {
        Protocol::HttpJson => {
            let json_struct = serde_json::to_string_pretty(&req).unwrap();
            buf = json_struct.into();
            ctype = "application/json";    
        },
        _ => {
            buf = req.encode_to_vec();
            ctype = "application/x-protobuf";    
        },
    };
    Ok((buf, ctype))
}

#[cfg(not(any(feature = "http-proto", feature = "http-json")))]
fn build_body(spans: Vec<SpanData>) -> TraceResult<(Vec<u8>, &'static str)> {
    Err(TraceError::Other(
        "No http protocol configured. Enable one via `http-proto` or `http-json`".into(),
    ))
}
