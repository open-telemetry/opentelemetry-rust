//! Tracez implementation
//!
pub(crate) use aggregator::SpanAggregator;
use opentelemetry::sdk::export::trace::SpanData;
pub use span_processor::ZPagesProcessor;
use futures::channel::oneshot;
use crate::proto::tracez::{TracezCounts, LatencyData, RunningData, ErrorData};

mod aggregator;
mod span_processor;
pub mod span_queue;

/// Message that used to pass commend between web servers, aggregators and span processors.
#[derive(Debug)]
pub enum TracezMessage {
    /// Sample span on start
    SampleSpan(SpanData),
    /// Span ended
    SpanEnd(SpanData),
    /// Shut down the aggregator
    ShutDown,
    Query {
        query: TracezQuery,
        response_tx: oneshot::Sender<TracezResponse>,
    },
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

pub enum TracezResponse {
    Aggregation(Vec<TracezCounts>),
    Latency(Vec<LatencyData>),
    Running(Vec<RunningData>),
    ErrorData(Vec<ErrorData>),
}

impl TracezResponse {
    /// Take the response and convert it into HTML page with pre-defined
    /// css styles for zPage.
    pub fn into_html(self) -> String {
        unimplemented!()
    }
    //todo: add into_json when RESTful APIs are available.
}