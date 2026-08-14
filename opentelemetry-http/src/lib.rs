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

impl Injector for HeaderInjector<'_> {
    /// Set a key and value in the HeaderMap.  Does nothing if the key or value are not valid inputs.
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
        }
    }

    /// Reserves capacity for at least `additional` more entries to be inserted.
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

/// Helper for extracting headers from HTTP Requests. This is used for OpenTelemetry context
/// propagation over HTTP.
/// See [this](https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/tracing-http-propagator/README.md)
/// for example usage.
pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl Extractor for HeaderExtractor<'_> {
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

    /// Get all the values for a key from the HeaderMap
    fn get_all(&self, key: &str) -> Option<Vec<&str>> {
        let all_iter = self.0.get_all(key).iter();
        if let (0, Some(0)) = all_iter.size_hint() {
            return None;
        }

        Some(all_iter.filter_map(|value| value.to_str().ok()).collect())
    }
}

pub type HttpError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Maximum bytes read from an HTTP response body, protecting against memory
/// exhaustion from a misbehaving endpoint. 4 MB matches the OpenTelemetry
/// .NET SDK.
#[cfg(any(feature = "reqwest", feature = "hyper"))]
const MAX_RESPONSE_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Initial body buffer capacity: the content-length hint capped at
/// [`MAX_RESPONSE_BODY_BYTES`] so a hostile hint cannot force a large allocation.
#[cfg(any(feature = "reqwest", feature = "hyper"))]
fn initial_body_capacity(content_length: Option<u64>) -> usize {
    let hint = content_length.unwrap_or(0);
    usize::try_from(hint)
        .unwrap_or(MAX_RESPONSE_BODY_BYTES)
        .min(MAX_RESPONSE_BODY_BYTES)
}

/// A minimal interface necessary for sending requests over HTTP.
/// Used primarily for exporting telemetry over HTTP. Also used for fetching
/// sampling strategies for JaegerRemoteSampler
///
/// Users sometime choose HTTP clients that relay on a certain async runtime. This trait allows
/// users to bring their choice of HTTP client.
#[async_trait]
pub trait HttpClient: Debug + Send + Sync {
    /// Send the specified HTTP request with `Vec<u8>` payload
    ///
    /// Returns the HTTP response including the status code and body.
    ///
    /// Returns an error if it can't connect to the server or the request could not be completed,
    /// e.g. because of a timeout, infinite redirects, or a loss of connection.
    #[deprecated(note = "Use `send_bytes` with `Bytes` payload instead.")]
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
        self.send_bytes(request.map(Into::into)).await
    }

    /// Send the specified HTTP request with `Bytes` payload.
    ///
    /// Returns the HTTP response including the status code and body.
    ///
    /// Returns an error if it can't connect to the server or the request could not be completed,
    /// e.g. because of a timeout, infinite redirects, or a loss of connection.
    async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError>;
}

#[cfg(feature = "reqwest")]
mod reqwest {
    use opentelemetry::otel_debug;

    use super::{async_trait, Bytes, HttpClient, HttpError, Request, Response};

    #[async_trait]
    impl HttpClient for reqwest::Client {
        async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
            otel_debug!(name: "ReqwestClient.Send");
            let request = request.try_into()?;
            let mut response = self.execute(request).await?.error_for_status()?;
            let status = response.status();
            let headers = std::mem::take(response.headers_mut());

            // Truncation is not an error: the exporter must not retry
            // because a response body was too large.
            #[cfg(not(target_arch = "wasm32"))]
            let body = {
                let mut body = bytes::BytesMut::with_capacity(super::initial_body_capacity(
                    response.content_length(),
                ));
                while let Some(chunk) = response.chunk().await? {
                    if body.len() + chunk.len() > super::MAX_RESPONSE_BODY_BYTES {
                        otel_debug!(
                            name: "ReqwestClient.ResponseBodyTruncated",
                            max_response_body_bytes = super::MAX_RESPONSE_BODY_BYTES as u64
                        );
                        break;
                    }
                    body.extend_from_slice(&chunk);
                }
                body.freeze()
            };
            // wasm reqwest has no incremental read; truncate after the fact.
            #[cfg(target_arch = "wasm32")]
            let body = {
                let mut body = response.bytes().await?;
                if body.len() > super::MAX_RESPONSE_BODY_BYTES {
                    otel_debug!(
                        name: "ReqwestClient.ResponseBodyTruncated",
                        max_response_body_bytes = super::MAX_RESPONSE_BODY_BYTES as u64
                    );
                    body = body.slice(..super::MAX_RESPONSE_BODY_BYTES);
                }
                body
            };

            let mut http_response = Response::builder().status(status).body(body)?;
            *http_response.headers_mut() = headers;

            Ok(http_response)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "reqwest-blocking")]
    #[async_trait]
    impl HttpClient for reqwest::blocking::Client {
        async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
            otel_debug!(name: "ReqwestBlockingClient.Send");
            let request = request.try_into()?;
            let mut response = self.execute(request)?.error_for_status()?;
            let status = response.status();
            let capacity = super::initial_body_capacity(response.content_length());
            let headers = std::mem::take(response.headers_mut());

            // Read one byte past the limit so truncation is detectable.
            let mut body = Vec::with_capacity(capacity);
            let mut limited =
                std::io::Read::take(response, super::MAX_RESPONSE_BODY_BYTES as u64 + 1);
            std::io::Read::read_to_end(&mut limited, &mut body)?;
            if body.len() > super::MAX_RESPONSE_BODY_BYTES {
                otel_debug!(
                    name: "ReqwestBlockingClient.ResponseBodyTruncated",
                    max_response_body_bytes = super::MAX_RESPONSE_BODY_BYTES as u64
                );
                body.truncate(super::MAX_RESPONSE_BODY_BYTES);
            }

            let mut http_response = Response::builder().status(status).body(Bytes::from(body))?;
            *http_response.headers_mut() = headers;

            Ok(http_response)
        }
    }
}

#[cfg(feature = "hyper")]
pub mod hyper {
    use super::{async_trait, Bytes, HttpClient, HttpError, Request, Response};
    use crate::ResponseExt;
    use http::HeaderValue;
    use http_body_util::{BodyExt, Full};
    use hyper::body::{Body as HttpBody, Frame};
    use hyper_util::client::legacy::{
        connect::{Connect, HttpConnector},
        Client,
    };
    use opentelemetry::otel_debug;
    use std::fmt::Debug;
    use std::pin::Pin;
    use std::task::{self, Poll};
    use std::time::Duration;
    use tokio::time;

    #[derive(Debug, Clone)]
    pub struct HyperClient<C = HttpConnector>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        inner: Client<C, Body>,
        timeout: Duration,
        authorization: Option<HeaderValue>,
    }

    impl<C> HyperClient<C>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        pub fn new(connector: C, timeout: Duration, authorization: Option<HeaderValue>) -> Self {
            // TODO - support custom executor
            let inner = Client::builder(hyper_util::rt::TokioExecutor::new()).build(connector);
            Self {
                inner,
                timeout,
                authorization,
            }
        }
    }

    impl HyperClient<HttpConnector> {
        /// Creates a new `HyperClient` with a default `HttpConnector`.
        pub fn with_default_connector(
            timeout: Duration,
            authorization: Option<HeaderValue>,
        ) -> Self {
            Self::new(HttpConnector::new(), timeout, authorization)
        }
    }

    #[async_trait]
    impl<C> HttpClient for HyperClient<C>
    where
        C: Connect + Clone + Send + Sync + 'static,
        HyperClient<C>: Debug,
    {
        async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
            otel_debug!(name: "HyperClient.Send");
            let (parts, body) = request.into_parts();
            let mut request = Request::from_parts(parts, Body(Full::from(body)));
            if let Some(ref authorization) = self.authorization {
                request
                    .headers_mut()
                    .insert(http::header::AUTHORIZATION, authorization.clone());
            }
            let mut response = time::timeout(self.timeout, self.inner.request(request)).await??;
            let status = response.status();
            let headers = std::mem::take(response.headers_mut());

            // Non-success responses only need the status; skip the body.
            if !status.is_success() {
                let mut http_response = Response::builder().status(status).body(Bytes::new())?;
                *http_response.headers_mut() = headers;
                return http_response.error_for_status();
            }

            let mut body = response.into_body();
            let size_hint = HttpBody::size_hint(&body);
            let mut collected = bytes::BytesMut::with_capacity(super::initial_body_capacity(
                size_hint.exact().or(size_hint.upper()),
            ));
            while let Some(frame) = body.frame().await {
                if let Ok(data) = frame?.into_data() {
                    if collected.len() + data.len() > super::MAX_RESPONSE_BODY_BYTES {
                        otel_debug!(
                            name: "HyperClient.ResponseBodyTruncated",
                            max_response_body_bytes = super::MAX_RESPONSE_BODY_BYTES as u64
                        );
                        break;
                    }
                    collected.extend_from_slice(&data);
                }
            }

            let mut http_response = Response::builder()
                .status(status)
                .body(collected.freeze())?;
            *http_response.headers_mut() = headers;

            Ok(http_response)
        }
    }

    pub struct Body(Full<Bytes>);

    impl HttpBody for Body {
        type Data = Bytes;
        type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

        #[inline]
        fn poll_frame(
            self: Pin<&mut Self>,
            cx: &mut task::Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            let inner_body = unsafe { self.map_unchecked_mut(|b| &mut b.0) };
            inner_body.poll_frame(cx).map_err(Into::into)
        }

        #[inline]
        fn is_end_stream(&self) -> bool {
            self.0.is_end_stream()
        }

        #[inline]
        fn size_hint(&self) -> hyper::body::SizeHint {
            self.0.size_hint()
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
    use http::HeaderValue;

    #[cfg(any(feature = "reqwest", feature = "hyper"))]
    #[test]
    fn initial_body_capacity_is_capped() {
        assert_eq!(initial_body_capacity(None), 0);
        assert_eq!(initial_body_capacity(Some(1024)), 1024);
        assert_eq!(
            initial_body_capacity(Some(u64::MAX)),
            MAX_RESPONSE_BODY_BYTES
        );
    }

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
    fn http_headers_get_all() {
        let mut carrier = http::HeaderMap::new();
        carrier.append("headerName", HeaderValue::from_static("value"));
        carrier.append("headerName", HeaderValue::from_static("value2"));
        carrier.append("headerName", HeaderValue::from_static("value3"));

        assert_eq!(
            HeaderExtractor(&carrier).get_all("HEADERNAME"),
            Some(vec!["value", "value2", "value3"]),
            "all values from a key extraction"
        )
    }

    #[test]
    fn http_headers_get_all_missing_key() {
        let mut carrier = http::HeaderMap::new();
        carrier.append("headerName", HeaderValue::from_static("value"));

        assert_eq!(
            HeaderExtractor(&carrier).get_all("not_existing"),
            None,
            "all values from a missing key extraction"
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

    #[test]
    fn http_headers_reserve() {
        let mut carrier = http::HeaderMap::new();

        // Test that reserve doesn't panic and works correctly
        {
            let mut injector = HeaderInjector(&mut carrier);
            injector.reserve(10);

            // Verify the HeaderMap still works after reserve
            injector.set("test-header", "test-value".to_string());
        }
        assert_eq!(
            HeaderExtractor(&carrier).get("test-header"),
            Some("test-value")
        );

        // Test reserve with zero capacity
        {
            let mut injector = HeaderInjector(&mut carrier);
            injector.reserve(0);
            injector.set("another-header", "another-value".to_string());
        }
        assert_eq!(
            HeaderExtractor(&carrier).get("another-header"),
            Some("another-value")
        );

        // Test that capacity is actually reserved (at least the requested amount)
        let mut new_carrier = http::HeaderMap::new();
        {
            let mut new_injector = HeaderInjector(&mut new_carrier);
            new_injector.reserve(5);
        }
        let initial_capacity = new_carrier.capacity();

        // Add some headers and verify capacity doesn't decrease
        {
            let mut new_injector = HeaderInjector(&mut new_carrier);
            for i in 0..3 {
                new_injector.set(&format!("header-{}", i), format!("value-{}", i));
            }
        }

        assert!(new_carrier.capacity() >= initial_capacity);
        assert!(new_carrier.capacity() >= 5);
    }
}

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    any(feature = "reqwest", feature = "hyper")
))]
mod bounded_body_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};

    const OVERSIZED_BODY_BYTES: usize = MAX_RESPONSE_BODY_BYTES + 64 * 1024;

    fn read_request_headers(stream: &mut std::net::TcpStream) {
        let mut buf = [0u8; 4096];
        let mut request = Vec::new();
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf).unwrap();
            assert!(n > 0, "peer closed before end of request headers");
            request.extend_from_slice(&buf[..n]);
        }
    }

    fn spawn_oversized_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_request_headers(&mut stream);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-length: {OVERSIZED_BODY_BYTES}\r\nconnection: close\r\n\r\n"
            )
            .unwrap();
            // Ignore write errors: the client may close once it hits the cap.
            let _ = stream.write_all(&vec![0u8; OVERSIZED_BODY_BYTES]);
        });
        addr
    }

    fn get_request(addr: SocketAddr) -> Request<Bytes> {
        Request::builder()
            .method("GET")
            .uri(format!("http://{addr}/"))
            .body(Bytes::new())
            .unwrap()
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn reqwest_truncates_oversized_response_body() {
        let addr = spawn_oversized_server();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let response = rt
            .block_on(async {
                let client = ::reqwest::Client::new();
                client.send_bytes(get_request(addr)).await
            })
            .unwrap();
        assert!(response.body().len() <= MAX_RESPONSE_BODY_BYTES);
        assert!(response.body().len() < OVERSIZED_BODY_BYTES);
        assert!(!response.body().is_empty());
    }

    #[cfg(all(feature = "reqwest", feature = "reqwest-blocking"))]
    #[test]
    fn reqwest_blocking_truncates_oversized_response_body() {
        let addr = spawn_oversized_server();
        let client = ::reqwest::blocking::Client::new();
        let response = futures_executor::block_on(client.send_bytes(get_request(addr))).unwrap();
        assert_eq!(response.body().len(), MAX_RESPONSE_BODY_BYTES);
    }

    #[cfg(feature = "hyper")]
    #[test]
    fn hyper_truncates_oversized_response_body() {
        let addr = spawn_oversized_server();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let response = rt
            .block_on(async {
                let client = crate::hyper::HyperClient::with_default_connector(
                    std::time::Duration::from_secs(10),
                    None,
                );
                client.send_bytes(get_request(addr)).await
            })
            .unwrap();
        assert!(response.body().len() <= MAX_RESPONSE_BODY_BYTES);
        assert!(response.body().len() < OVERSIZED_BODY_BYTES);
        assert!(!response.body().is_empty());
    }

    #[cfg(feature = "hyper")]
    #[test]
    fn hyper_does_not_read_error_response_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_request_headers(&mut stream);
            write!(
                stream,
                "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 10\r\nconnection: close\r\n\r\n0123456789"
            )
            .unwrap();
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(async {
            let client = crate::hyper::HyperClient::with_default_connector(
                std::time::Duration::from_secs(10),
                None,
            );
            client.send_bytes(get_request(addr)).await
        });
        let error = result.expect_err("non-success status must surface as an error");
        assert!(error.to_string().contains("500"));
    }
}
