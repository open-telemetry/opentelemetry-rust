#[cfg(feature = "reqwest")]
use std::convert::TryInto;
#[cfg(any(feature = "surf", feature = "reqwest"))]
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use http::Request;
use opentelemetry::propagation::{Extractor, Injector};
#[cfg(any(feature = "surf", feature = "reqwest"))]
use opentelemetry::sdk::export::ExportError;
use opentelemetry::trace::TraceError;

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

/// A minimal interface necessary for export spans over HTTP.
///
/// Users sometime choose http clients that relay on certain runtime. This trait
/// allows users to bring their choice of http clients.
#[async_trait]
pub trait HttpClient: Debug + Send + Sync {
    /// Send a batch of spans to collectors
    async fn send(&self, request: Request<Vec<u8>>) -> Result<(), TraceError>;
}

#[cfg(feature = "reqwest")]
#[derive(Debug)]
struct ReqwestError(reqwest::Error);

#[cfg(feature = "reqwest")]
impl Display for ReqwestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[cfg(feature = "reqwest")]
impl std::error::Error for ReqwestError {}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ReqwestError {
    fn from(err: reqwest::Error) -> Self {
        ReqwestError(err)
    }
}

#[cfg(feature = "reqwest")]
impl ExportError for ReqwestError {
    fn exporter_name(&self) -> &'static str {
        "reqwest"
    }
}

#[cfg(feature = "surf")]
impl ExportError for SurfError {
    fn exporter_name(&self) -> &'static str {
        "surf"
    }
}

#[cfg(feature = "surf")]
#[derive(Debug)]
struct SurfError(surf::Error);

#[cfg(feature = "surf")]
impl Display for SurfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[cfg(feature = "surf")]
impl std::error::Error for SurfError {}

#[cfg(feature = "surf")]
impl From<surf::Error> for SurfError {
    fn from(err: surf::Error) -> Self {
        SurfError(err)
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HttpClient for reqwest::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<(), TraceError> {
        let request = request.try_into().map_err(ReqwestError::from)?;
        let _ = self
            .execute(request)
            .await
            .and_then(|rsp| rsp.error_for_status())
            .map_err(ReqwestError::from)?;
        Ok(())
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HttpClient for reqwest::blocking::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<(), TraceError> {
        let _ = request
            .try_into()
            .and_then(|req| self.execute(req))
            .and_then(|rsp| rsp.error_for_status())
            .map_err(ReqwestError::from)?;
        Ok(())
    }
}

#[cfg(feature = "surf")]
#[async_trait]
impl HttpClient for surf::Client {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<(), TraceError> {
        let (parts, body) = request.into_parts();
        let uri = parts
            .uri
            .to_string()
            .parse()
            .map_err(|_err: surf::http::url::ParseError| TraceError::from("error parse url"))?;

        let req = surf::Request::builder(surf::http::Method::Post, uri)
            .content_type("application/json")
            .body(body);
        let result = self.send(req).await.map_err::<SurfError, _>(Into::into)?;

        if result.status().is_success() {
            Ok(())
        } else {
            Err(SurfError(surf::Error::from_str(
                result.status(),
                result.status().canonical_reason(),
            ))
            .into())
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
