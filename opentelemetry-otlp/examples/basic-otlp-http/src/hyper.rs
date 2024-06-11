use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use hyper::{
    client::{connect::Connect, HttpConnector},
    Body, Client,
};
use opentelemetry_http::{HttpClient, HttpError, ResponseExt};

pub struct HyperClient<C> {
    inner: hyper::Client<C>,
}

impl Default for HyperClient<HttpConnector> {
    fn default() -> Self {
        Self {
            inner: Client::new(),
        }
    }
}

impl<C> std::fmt::Debug for HyperClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperClient")
            .field("inner", &self.inner)
            .finish()
    }
}

#[async_trait]
impl<C: Connect + Clone + Send + Sync + 'static> HttpClient for HyperClient<C> {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
        let request = request.map(Body::from);

        let (parts, body) = self
            .inner
            .request(request)
            .await?
            .error_for_status()?
            .into_parts();
        let body = hyper::body::to_bytes(body).await?;

        Ok(Response::from_parts(parts, body))
    }
}
