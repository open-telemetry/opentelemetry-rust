use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper_util::{
    client::legacy::{
        connect::{Connect, HttpConnector},
        Client,
    },
    rt::TokioExecutor,
};
use opentelemetry_http::{HttpClient, HttpError, ResponseExt};

pub struct HyperClient<C> {
    inner: hyper_util::client::legacy::Client<C, Full<Bytes>>,
}

impl Default for HyperClient<HttpConnector> {
    fn default() -> Self {
        Self {
            inner: Client::builder(TokioExecutor::new()).build_http(),
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
        let request = request.map(|body| Full::new(Bytes::from(body)));

        let (parts, body) = self
            .inner
            .request(request)
            .await?
            .error_for_status()?
            .into_parts();
        let body = body.collect().await?.to_bytes();

        Ok(Response::from_parts(parts, body))
    }
}
