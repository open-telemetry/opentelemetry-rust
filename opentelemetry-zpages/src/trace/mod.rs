//! Tracez implementation
//!
use opentelemetry_proto::tonic::tracez::v1::{ErrorData, LatencyData, RunningData, TracezCounts};

use async_channel::{SendError, Sender};
use futures_channel::oneshot::{self, Canceled};
use opentelemetry::runtime::Runtime;
use opentelemetry::sdk::export::trace::SpanData;
use serde::ser::SerializeSeq;
use serde::Serializer;
use std::fmt::Formatter;
use std::sync::Arc;

mod aggregator;
pub(crate) mod span_processor;
pub(crate) mod span_queue;

/// Create tracez components. This function will return a [`ZPagesSpanProcessor`] that should be installed
/// into the [`TracerProvider`] and a [`TracezQuerier`] for http server to access the aggregated
/// information on spans.
///
/// The `sample_size` config how may spans to sample for each unique span name.
///
/// [`ZPagesSpanProcessor`]: span_processor::ZPagesSpanProcessor
/// [`TracerProvider`]: opentelemetry::trace::TracerProvider
///
/// ## Example
/// ```no_run
/// # use opentelemetry_zpages::tracez;
/// # use opentelemetry::{global, runtime::Tokio, sdk::trace, trace::Tracer};
/// # use std::sync::Arc;
/// # fn main() {
///     let (processor, querier) = tracez(5, Tokio); // sample 5 spans for each unique span name
///     let provider = trace::TracerProvider::builder()
///         .with_span_processor(processor)
///         .build();
///     global::set_tracer_provider(provider);
///
///     // use querier to retrieve the aggregated span information
/// # }
///
/// ```
pub fn tracez<R: Runtime>(
    sample_size: usize,
    runtime: R,
) -> (span_processor::ZPagesSpanProcessor, TracezQuerier) {
    let (tx, rx) = async_channel::unbounded();
    let span_processor = span_processor::ZPagesSpanProcessor::new(tx.clone());
    let mut aggregator = aggregator::SpanAggregator::new(rx, sample_size);
    runtime.spawn(Box::pin(async move {
        aggregator.process().await;
    }));
    (span_processor, TracezQuerier(Arc::new(tx)))
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

impl std::fmt::Debug for TracezMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TracezMessage::SampleSpan(_) => f.write_str("span starts"),
            TracezMessage::SpanEnd(_) => f.write_str("span ends"),
            TracezMessage::ShutDown => f.write_str("shut down"),
            TracezMessage::Query { .. } => f.write_str("query aggregation results"),
        }
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

/// Provide wrapper functions to query the aggregated span info.
// TracezQuerier creates the oneshot channel and send the TracezMessage to the SpanAggregator.
#[derive(Clone, Debug)]
pub struct TracezQuerier(Arc<Sender<TracezMessage>>);

impl TracezQuerier {
    /// Return the aggregation status for spans.
    ///
    /// The aggregation will contains the error, running and latency counts for all span name
    /// groupings.
    pub async fn aggregation(&self) -> Result<TracezResponse, TracezError> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(TracezMessage::Query {
                query: TracezQuery::Aggregation,
                response_tx: tx,
            })
            .await?;
        rx.await.map_err::<TracezError, _>(Into::into)?
    }

    /// Return the sample spans for the given bucket index.
    pub async fn latency(
        &self,
        bucket_index: usize,
        span_name: String,
    ) -> Result<TracezResponse, TracezError> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(TracezMessage::Query {
                query: TracezQuery::Latency {
                    bucket_index,
                    span_name,
                },
                response_tx: tx,
            })
            .await?;
        rx.await.map_err::<TracezError, _>(Into::into)?
    }

    /// Return the sample running spans' snapshot.
    ///
    /// Note that current implementation cannot include the changes made to spans after the spans
    /// started. For example, the events added or the links added.
    pub async fn running(&self, span_name: String) -> Result<TracezResponse, TracezError> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(TracezMessage::Query {
                query: TracezQuery::Running { span_name },
                response_tx: tx,
            })
            .await?;
        rx.await.map_err::<TracezError, _>(Into::into)?
    }

    /// Return the sample spans with error status.
    pub async fn error(&self, span_name: String) -> Result<TracezResponse, TracezError> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(TracezMessage::Query {
                query: TracezQuery::Error { span_name },
                response_tx: tx,
            })
            .await?;
        rx.await.map_err::<TracezError, _>(Into::into)?
    }
}

impl Drop for TracezQuerier {
    fn drop(&mut self) {
        // shut down aggregator if it is still running
        let _ = self.0.try_send(TracezMessage::ShutDown);
    }
}

/// Tracez API's error.
#[derive(Debug)]
pub enum TracezError {
    /// There isn't a valid tracez operation for that API
    InvalidArgument {
        /// Describe the operation on the tracez
        api: &'static str,
        /// Error message
        message: &'static str,
    },
    /// Operation cannot be found
    NotFound {
        /// Describe the operation on the tracez
        api: &'static str,
    },
    /// Error when serialize the TracezResponse to json.
    Serialization,
    /// The span aggregator has been dropped.
    AggregatorDropped,
}

impl From<Canceled> for TracezError {
    fn from(_: Canceled) -> Self {
        TracezError::AggregatorDropped
    }
}

impl From<async_channel::SendError<TracezMessage>> for TracezError {
    fn from(_: SendError<TracezMessage>) -> Self {
        // Since we employed a unbounded channel to send message to aggregator.
        // The only reason why the send would return errors is the receiver has closed
        // This should only happen if the span aggregator has been dropped.
        TracezError::AggregatorDropped
    }
}

impl std::fmt::Display for TracezError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TracezError::InvalidArgument { api: _, message } => f.write_str(message),
            TracezError::NotFound { api: _ } => {
                f.write_str("the requested resource is not founded")
            }
            TracezError::Serialization => f.write_str("cannot serialize the response into json"),
            TracezError::AggregatorDropped => {
                f.write_str("the span aggregator is already dropped when querying")
            }
        }
    }
}

impl TracezResponse {
    /// Convert the `TracezResponse` into json.
    ///
    /// Throw a `TracezError` if the serialization fails.
    #[cfg(feature = "with-serde")]
    pub fn into_json(self) -> Result<String, TracezError> {
        serde_json::to_string(&self).map_err(|_| TracezError::Serialization)
    }
}
