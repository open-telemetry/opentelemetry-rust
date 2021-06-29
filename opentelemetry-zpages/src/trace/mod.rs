//! Tracez implementation
//!
use crate::proto::tracez::{ErrorData, LatencyData, RunningData, TracezCounts};

use futures::channel::oneshot;
use opentelemetry::sdk::export::trace::SpanData;

use std::fmt::Formatter;

mod aggregator;
pub mod span_processor;
pub(crate) mod span_queue;

/// Message that used to pass commend between web servers, aggregators and span processors.
pub enum TracezMessage {
    /// Sample span on start
    SampleSpan(SpanData),
    /// Span ended
    SpanEnd(SpanData),
    /// Shut down the aggregator
    ShutDown,
    /// Run a query from the web service
    Query {
        /// Query content
        query: TracezQuery,
        /// Channel to send the response
        response_tx: oneshot::Sender<Result<TracezResponse, QueryError>>,
    },
}

impl std::fmt::Debug for TracezMessage {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

/// Tracez APIs.
/// As defined in [spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/experimental/trace/zpages.md#http-server)
#[derive(Debug)]
pub enum TracezQuery {
    /// tracez/api/aggregations
    Aggregation,
    /// tracez/api/latency/{bucket_index}/{span_name}
    Latency {
        /// index of the bucket in API path
        bucket_index: usize,
        /// span name in API path
        span_name: String,
    },
    /// tracez/api/running/{span_name}
    Running {
        /// span name in API path
        span_name: String,
    },
    /// tracez/api/error/{span_name}
    Error {
        /// span name in API path
        span_name: String,
    },
}

/// Tracez APIs' response
#[derive(Debug)]
pub enum TracezResponse {
    /// tracez/api/aggregations
    Aggregation(Vec<TracezCounts>),
    /// tracez/api/latency/{bucket_index}/{span_name}
    Latency(Vec<LatencyData>),
    /// tracez/api/running/{span_name}
    Running(Vec<RunningData>),
    /// tracez/api/error/{span_name}
    ErrorData(Vec<ErrorData>),
}

/// Tracez API's error.
#[derive(Debug)]
pub enum QueryError {
    InvalidArgument {
        api: &'static str,
        message: &'static str,
    },
    NotFound {
        api: &'static str,
    },
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::InvalidArgument { api: _, message } => f.write_str(message),
            QueryError::NotFound { api: _ } => f.write_str("the requested resource is not founded"),
        }
    }
}

impl TracezResponse {
    /// Take the response and convert it into HTML page with pre-defined
    /// css styles for zPage.
    pub fn into_html(self) -> String {
        unimplemented!()
    }
    //todo: add into_json when RESTful APIs are available.
}
