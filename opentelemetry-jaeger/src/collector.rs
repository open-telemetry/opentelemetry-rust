//! # HTTP Jaeger Collector Client
use crate::thrift::jaeger;
use http::{Request, Uri};
use isahc::{
    auth::{Authentication, Credentials},
    config::Configurable,
    HttpClient,
};
use std::io::{self, Cursor};
use std::sync::atomic::{AtomicUsize, Ordering};
use thrift::protocol::TBinaryOutputProtocol;

/// `CollectorAsyncClientHttp` implements an async version of the
/// `TCollectorSyncClient` interface over HTTP
#[derive(Debug)]
pub(crate) struct CollectorAsyncClientHttp {
    endpoint: Uri,
    client: HttpClient,
    payload_size_estimate: AtomicUsize,
}

impl CollectorAsyncClientHttp {
    /// Create a new HTTP collector client
    pub(crate) fn new(
        endpoint: Uri,
        username: Option<String>,
        password: Option<String>,
    ) -> thrift::Result<Self> {
        let mut builder = HttpClient::builder();
        if let (Some(username), Some(password)) = (username, password) {
            builder = builder
                .authentication(Authentication::basic())
                .credentials(Credentials::new(username, password));
        }
        let client = builder
            .build()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        let payload_size_estimate = AtomicUsize::new(512);

        Ok(CollectorAsyncClientHttp {
            endpoint,
            client,
            payload_size_estimate,
        })
    }

    /// Submit list of Jaeger batches
    pub(crate) async fn submit_batch(
        &self,
        batch: jaeger::Batch,
    ) -> thrift::Result<jaeger::BatchSubmitResponse> {
        // estimate transport capacity based on last request
        let estimate = self.payload_size_estimate.load(Ordering::Relaxed);

        // Write payload to transport buffer
        let transport = Cursor::new(Vec::with_capacity(estimate));
        let mut protocol = TBinaryOutputProtocol::new(transport, true);
        batch.write_to_out_protocol(&mut protocol)?;

        // Use current batch capacity as new estimate
        self.payload_size_estimate
            .store(protocol.transport.get_ref().len(), Ordering::Relaxed);

        // Build collector request
        let req = Request::builder()
            .method("POST")
            .uri(&self.endpoint)
            .header("Content-Type", "application/vnd.apache.thrift.binary")
            .body(protocol.transport.into_inner())
            .expect("request should always be valid");

        // Send request to collector
        let res = self
            .client
            .send_async(req)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        if !res.status().is_success() {
            return Err(thrift::Error::from(io::Error::new(
                io::ErrorKind::Other,
                format!("Expected success response, got {:?}", res.status()),
            )));
        }

        Ok(jaeger::BatchSubmitResponse { ok: true })
    }
}
