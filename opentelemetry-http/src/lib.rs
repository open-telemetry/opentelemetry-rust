use async_trait::async_trait;
use std::fmt::Debug;

#[doc(no_inline)]
pub use bytes::Bytes;
#[doc(no_inline)]
pub use http::{Request, Response};
use opentelemetry::propagation::{Extractor, Injector};

/// Helper for injecting headers into HTTP Requests. This is used for OpenTelemetry context
/// propagation over HTTP.
/// See [this](https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/tracing-http-propagator/README.md)
/// for example usage.
pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    /// Set a key and value in the HeaderMap.  Does nothing if the key or value are not valid inputs.
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
        }
    }
}

/// Helper for extracting headers from HTTP Requests. This is used for OpenTelemetry context
/// propagation over HTTP.
/// See [this](https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/tracing-http-propagator/README.md)
/// for example usage.
pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the HeaderMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
    }
}

pub type HttpError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A minimal interface necessary for sending requests over HTTP.
/// Used primarily for exporting telemetry over HTTP. Also used for fetching
/// sampling strategies for JaegerRemoteSampler
///
/// Users sometime choose HTTP clients that relay on a certain async runtime. This trait allows
/// users to bring their choice of HTTP client.
#[async_trait]
pub trait HttpClient: Debug + Send + Sync {
    /// Send the specified HTTP request
    ///
    /// Returns the HTTP response including the status code and body.
    ///
    /// Returns an error if it can't connect to the server or the request could not be completed,
    /// e.g. because of a timeout, infinite redirects, or a loss of connection.
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError>;
}

#[cfg(feature = "reqwest")]
mod reqwest {
    use super::{async_trait, Bytes, HttpClient, HttpError, Request, Response};

    #[async_trait]
    impl HttpClient for reqwest::Client {
        async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
            let request = request.try_into()?;
            let mut response = self.execute(request).await?.error_for_status()?;
            let headers = std::mem::take(response.headers_mut());
            let mut http_response = Response::builder()
                .status(response.status())
                .body(response.bytes().await?)?;
            *http_response.headers_mut() = headers;

            Ok(http_response)
        }
    }

    #[async_trait]
    impl HttpClient for reqwest::blocking::Client {
        async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
            let request = request.try_into()?;
            let mut response = self.execute(request)?.error_for_status()?;
            let headers = std::mem::take(response.headers_mut());
            let mut http_response = Response::builder()
                .status(response.status())
                .body(response.bytes()?)?;
            *http_response.headers_mut() = headers;

            Ok(http_response)
        }
    }
}

#[cfg(feature = "isahc")]
mod isahc {
    use crate::ResponseExt;

    use super::{async_trait, Bytes, HttpClient, HttpError, Request, Response};
    use isahc::AsyncReadResponseExt;
    use std::convert::TryInto as _;

    #[async_trait]
    impl HttpClient for isahc::HttpClient {
        async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
            let mut response = self.send_async(request).await?;
            let mut bytes = Vec::with_capacity(response.body().len().unwrap_or(0).try_into()?);
            response.copy_to(&mut bytes).await?;

            let headers = std::mem::take(response.headers_mut());
            let mut http_response = Response::builder()
                .status(response.status().as_u16())
                .body(bytes.into())?;
            *http_response.headers_mut() = headers;

            Ok(http_response.error_for_status()?)
        }
    }
}

#[cfg(any(feature = "hyper", feature = "hyper_tls"))]
pub mod hyper {
    use crate::ResponseExt;

    use super::{async_trait, Bytes, HttpClient, HttpError, Request, Response};
    use http::HeaderValue;
    use hyper::client::connect::Connect;
    use hyper::Client;
    use std::fmt::Debug;
    use std::time::Duration;
    use tokio::time;

    #[derive(Debug, Clone)]
    pub struct HyperClient<C> {
        inner: Client<C>,
        timeout: Duration,
        authorization: Option<HeaderValue>,
    }

    impl<C> HyperClient<C> {
        pub fn new_with_timeout(inner: Client<C>, timeout: Duration) -> Self {
            Self {
                inner,
                timeout,
                authorization: None,
            }
        }

        pub fn new_with_timeout_and_authorization_header(
            inner: Client<C>,
            timeout: Duration,
            authorization: HeaderValue,
        ) -> Self {
            Self {
                inner,
                timeout,
                authorization: Some(authorization),
            }
        }
    }

    #[async_trait]
    impl<C> HttpClient for HyperClient<C>
    where
        C: Connect + Send + Sync + Clone + Debug + 'static,
    {
        async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
            let (parts, body) = request.into_parts();
            let mut request = Request::from_parts(parts, body.into());
            if let Some(ref authorization) = self.authorization {
                request
                    .headers_mut()
                    .insert(http::header::AUTHORIZATION, authorization.clone());
            }
            let mut response = time::timeout(self.timeout, self.inner.request(request)).await??;
            let headers = std::mem::take(response.headers_mut());
            let mut http_response = Response::builder()
                .status(response.status())
                .body(hyper::body::to_bytes(response.into_body()).await?)?;
            *http_response.headers_mut() = headers;

            Ok(http_response.error_for_status()?)
        }
    }
}

/// Methods to make working with responses from the [`HttpClient`] trait easier.
pub trait ResponseExt: Sized {
    /// Turn a response into an error if the HTTP status does not indicate success (200 - 299).
    fn error_for_status(self) -> Result<Self, HttpError>;
}

impl<T> ResponseExt for Response<T> {
    fn error_for_status(self) -> Result<Self, HttpError> {
        if self.status().is_success() {
            Ok(self)
        } else {
            Err(format!("request failed with status {}", self.status()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_headers_get() {
        let mut carrier = http::HeaderMap::new();
        HeaderInjector(&mut carrier).set("headerName", "value".to_string());

        assert_eq!(
            HeaderExtractor(&carrier).get("HEADERNAME"),
            Some("value"),
            "case insensitive extraction"
        )
    }

    #[test]
    fn http_headers_keys() {
        let mut carrier = http::HeaderMap::new();
        HeaderInjector(&mut carrier).set("headerName1", "value1".to_string());
        HeaderInjector(&mut carrier).set("headerName2", "value2".to_string());

        let extractor = HeaderExtractor(&carrier);
        let got = extractor.keys();
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"headername1"));
        assert!(got.contains(&"headername2"));
    }
}
