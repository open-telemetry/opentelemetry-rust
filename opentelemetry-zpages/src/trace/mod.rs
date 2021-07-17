//! Tracez implementation
//!
use crate::proto::tracez::{ErrorData, LatencyData, RunningData, TracezCounts};

use async_channel::Sender;
use futures::channel::oneshot;
use opentelemetry::sdk::export::trace::SpanData;

use opentelemetry::runtime::Runtime;
use serde::ser::SerializeSeq;
use serde::Serializer;
use std::fmt::Formatter;

mod aggregator;
pub(crate) mod span_processor;
pub(crate) mod span_queue;

/// Create tracez components. This function will create a `ZPageSpanProcessor` and a `SpanAggregator`.
///
/// It will start the aggregator with `sample_size` sample spans for each unique span name.
///
/// Return a `SpanProcessor` that should be installed into the `TracerProvider` and a `Sender` to send
/// query requests and shutdown command.
pub fn tracez<R: Runtime>(
    sample_size: usize,
    runtime: R,
) -> (span_processor::ZPagesSpanProcessor, Sender<TracezMessage>) {
    let (tx, rx) = async_channel::unbounded();
    let span_processor = span_processor::ZPagesSpanProcessor::new(tx.clone());
    let mut aggregator = aggregator::SpanAggregator::new(rx, sample_size);
    let _ = runtime.spawn(Box::pin(async move {
        aggregator.process().await;
    }));
    (span_processor, tx)
}

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
        response_tx: oneshot::Sender<Result<TracezResponse, TracezError>>,
    },
}

impl TracezMessage {
    /// Create a message to shut down span aggregator
    pub fn shutdown() -> TracezMessage {
        TracezMessage::ShutDown
    }

    /// Create a message for aggregation API.
    ///
    /// Return the message and the `Receiver` to receive the response.
    pub fn aggregation() -> (
        TracezMessage,
        oneshot::Receiver<Result<TracezResponse, TracezError>>,
    ) {
        let (tx, rx) = oneshot::channel();
        (
            TracezMessage::Query {
                query: TracezQuery::Aggregation,
                response_tx: tx,
            },
            rx,
        )
    }

    /// Create a message for latency API
    ///
    /// Return the message and the `Receiver` to receive the response.
    pub fn latency(
        bucket_index: usize,
        span_name: String,
    ) -> (
        TracezMessage,
        oneshot::Receiver<Result<TracezResponse, TracezError>>,
    ) {
        let (tx, rx) = oneshot::channel();
        (
            TracezMessage::Query {
                query: TracezQuery::Latency {
                    bucket_index,
                    span_name,
                },
                response_tx: tx,
            },
            rx,
        )
    }

    /// Create a message for running spans API
    ///
    /// Return the message and the `Receiver` to receive the response.
    pub fn running(
        span_name: String,
    ) -> (
        TracezMessage,
        oneshot::Receiver<Result<TracezResponse, TracezError>>,
    ) {
        let (tx, rx) = oneshot::channel();
        (
            TracezMessage::Query {
                query: TracezQuery::Running { span_name },
                response_tx: tx,
            },
            rx,
        )
    }

    /// Create a message for error spans API
    ///
    /// Return the message and the `Receiver` to receive the response.
    pub fn error(
        span_name: String,
    ) -> (
        TracezMessage,
        oneshot::Receiver<Result<TracezResponse, TracezError>>,
    ) {
        let (tx, rx) = oneshot::channel();
        (
            TracezMessage::Query {
                query: TracezQuery::Error { span_name },
                response_tx: tx,
            },
            rx,
        )
    }
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
    Error(Vec<ErrorData>),
}

impl serde::Serialize for TracezResponse {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            TracezResponse::Aggregation(data) => {
                let mut list = serializer.serialize_seq(Some(data.len()))?;
                for e in data {
                    list.serialize_element(e)?;
                }
                list.end()
            }
            TracezResponse::Latency(data) => {
                let mut list = serializer.serialize_seq(Some(data.len()))?;
                for e in data {
                    list.serialize_element(e)?;
                }
                list.end()
            }
            TracezResponse::Running(data) => {
                let mut list = serializer.serialize_seq(Some(data.len()))?;
                for e in data {
                    list.serialize_element(e)?;
                }
                list.end()
            }
            TracezResponse::Error(data) => {
                let mut list = serializer.serialize_seq(Some(data.len()))?;
                for e in data {
                    list.serialize_element(e)?;
                }
                list.end()
            }
        }
    }
}

/// Tracez API's error.
#[derive(Debug)]
pub enum TracezError {
    InvalidArgument {
        api: &'static str,
        message: &'static str,
    },
    NotFound {
        api: &'static str,
    },
    Serialization,
}

impl std::fmt::Display for TracezError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TracezError::InvalidArgument { api: _, message } => f.write_str(message),
            TracezError::NotFound { api: _ } => {
                f.write_str("the requested resource is not founded")
            }
            TracezError::Serialization => f.write_str("cannot serialize the response into json"),
        }
    }
}

impl TracezResponse {
    /// Take the response and convert it into HTML page with pre-defined
    /// css styles for zPage.
    pub fn into_html(self) -> String {
        unimplemented!()
    }

    /// Convert the `TracezResponse` into json.
    ///
    /// Throw a `TracezError` if the serialization fails.
    #[cfg(feature = "with-serde")]
    pub fn into_json(self) -> Result<String, TracezError> {
        serde_json::to_string(&self).map_err(|_| TracezError::Serialization)
    }
}

#[cfg(test)]
mod tests {}
