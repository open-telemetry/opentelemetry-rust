use std::collections::HashMap;

use warp::{
    filters::header::headers_cloned,
    http::{
        header::{HeaderMap, HeaderValue},
        Method,
    },
    Filter, Rejection,
    path::FullPath,
};
use tracing::{event, instrument, Level};

/* Ports - Middleware */

type HttpResponse<T> = std::result::Result<T, Rejection>;

pub struct FromRequest {
    method: Method,
    query: HashMap<String, String>,
    path: FullPath,
}

impl FromRequest {
    pub fn new(
        method: Method,
        query: HashMap<String, String>,
        path: FullPath,
    ) -> Self {
        FromRequest { method, query, path }
    }

    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

// Represents an innocent intercept for requests, give superpowers to here!
pub fn with_interceptor() -> impl Filter<Extract = (FromRequest,), Error = Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| headers)
        .and(warp::method())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::path::full())
        .and_then(http_middleware)
}

// Just doing access control and instrumentation manually...
#[instrument]
pub async fn http_middleware(
    headers: HeaderMap<HeaderValue>,
    method: Method,
    query: HashMap<String, String>,
    path: FullPath,
) -> HttpResponse<FromRequest> {
    // TODO: intercept requests and document something...

    // Received from Request...
    let from_request = FromRequest::new(method.clone(), query, path);

    // Logging anything... or use your metrics here! =]
    event!(
        Level::INFO,
        "Received method: {}; PATH: {:?}; HEADERS: {:?}; QUERY: {:?}",
        from_request.method(), from_request.path(), headers.clone(), from_request.query(),
    );

    Ok(from_request)
}
