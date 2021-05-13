//! ## Span Aggregator
//!
//! Process the span information, aggregate counts for latency, running, and errors for spans grouped
//! by name.
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::channel::{mpsc, oneshot};
use futures::StreamExt;

use opentelemetry::trace::{SpanContext, StatusCode};

use crate::proto::tracez::TracezCounts;
use crate::trace::{TracezMessage, TracezQuery, TracezResponse};
use crate::SpanQueue;
use opentelemetry::sdk::export::trace::SpanData;

lazy_static! {
    static ref LATENCY_BUCKET: [Duration; 9] = [
        Duration::from_micros(0),
        Duration::from_micros(10),
        Duration::from_micros(100),
        Duration::from_millis(1),
        Duration::from_millis(10),
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(10),
        Duration::from_secs(100),
    ];
}
const LATENCY_BUCKET_COUNT: usize = 9;

/// Aggregate span information from zPage span processor and feed that information to server when
/// requested.
#[derive(Debug)]
pub(crate) struct SpanAggregator {
    receiver: mpsc::Receiver<TracezMessage>,
    summaries: HashMap<String, SpanSummary>,
    sample_size: usize,
}

impl SpanAggregator {
    /// Create a span aggregator
    pub(crate) fn new(
        receiver: mpsc::Receiver<TracezMessage>,
        sample_size: usize,
    ) -> SpanAggregator {
        SpanAggregator {
            receiver,
            summaries: HashMap::new(),
            sample_size,
        }
    }

    /// Process request from http server or the span processor.
    pub(crate) async fn process(&mut self) {
        loop {
            match self.receiver.next().await {
                None => {
                    // all senders have been dropped. Thus, close it
                    self.receiver.close();
                    return;
                }
                Some(msg) => {
                    match msg {
                        TracezMessage::ShutDown => {
                            self.receiver.close();
                            return;
                        }
                        TracezMessage::SpanEnd(span) => {
                            let summary = self
                                .summaries
                                .entry(span.name.clone().into())
                                .or_insert(SpanSummary::new(self.sample_size));

                            summary.running.remove(span.span_context.clone());

                            if span.status_code != StatusCode::Ok {
                                summary.error.add_span(span);
                            } else {
                                let latency_idx = latency_bucket(span.start_time, span.end_time);
                                summary
                                    .latencies
                                    .get_mut(latency_idx)
                                    .map(|stats| stats.add_span(span));
                            }
                        }
                        TracezMessage::SampleSpan(span) => {
                            // Resample span whenever there is a new span starts.
                            //
                            // This helps us clean the stale span that failed to be evicted because
                            // of the failure to deliver the span end messages.
                            let summary = self
                                .summaries
                                .entry(span.name.clone().into())
                                .or_insert(SpanSummary::new(self.sample_size));
                            summary.running.add_span(span)
                        }
                        TracezMessage::Query { query, response_tx } => {
                            self.handle_query(query, response_tx).await
                        }
                    }
                }
            }
        }
    }

    async fn handle_query(
        &mut self,
        query: TracezQuery,
        response_tx: oneshot::Sender<TracezResponse>,
    ) {
        match query {
            TracezQuery::Aggregation => {
                let _ = response_tx.send(TracezResponse::Aggregation(
                    self.summaries
                        .iter()
                        .map(|(span_name, summary)| TracezCounts {
                            spanname: span_name.clone(),
                            latency: summary
                                .latencies
                                .iter()
                                .map(|stats| stats.count as u32)
                                .collect(),
                            running: summary.running.count as u32,
                            error: summary.error.count as u32,
                            ..Default::default()
                        })
                        .collect(),
                ));
            }
            _ => {}
        }
    }
}

fn latency_bucket(start_time: SystemTime, end_time: SystemTime) -> usize {
    let latency = end_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        - start_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));
    for idx in 1..LATENCY_BUCKET.len() {
        if LATENCY_BUCKET[idx] > latency {
            return (idx - 1) as usize;
        }
    }
    return LATENCY_BUCKET.len() - 1;
}

#[derive(Debug, Clone)]
struct SpanStats {
    examples: SpanQueue,
    count: usize,
}

impl SpanStats {
    /// Create a new SpanStats
    fn new(sample_size: usize) -> SpanStats {
        SpanStats {
            examples: SpanQueue::new(sample_size),
            count: 0,
        }
    }
    /// Add a span into examples
    fn add_span(&mut self, span: SpanData) {
        self.count += 1;
        self.examples.push_back(span)
    }

    /// Remove a span from the examples
    fn remove(&mut self, context: SpanContext) -> Option<SpanData> {
        self.count -= 1;
        self.examples.remove(context)
    }
}

#[derive(Debug)]
struct SpanSummary {
    running: SpanStats,
    error: SpanStats,
    latencies: Vec<SpanStats>,
}

impl SpanSummary {
    fn new(sample_size: usize) -> SpanSummary {
        SpanSummary {
            running: SpanStats::new(sample_size),
            error: SpanStats::new(sample_size),
            latencies: vec![SpanStats::new(sample_size); LATENCY_BUCKET_COUNT],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use futures::channel::mpsc;
    use futures::SinkExt;

    use opentelemetry::trace::{SpanContext, SpanId, StatusCode, TraceId, TraceState};

    use crate::trace::{SpanAggregator, TracezMessage};
    use opentelemetry::sdk::export::trace::SpanData;
    use opentelemetry::testing::trace::new_test_export_span_data;
    use std::borrow::Cow;

    #[tokio::test]
    async fn test_process() -> Result<(), Box<dyn std::error::Error>> {
        let (mut sender, receiver) = mpsc::channel::<TracezMessage>(100);
        let mut aggregator = SpanAggregator::new(receiver, 5);

        let handle = tokio::spawn(async move {
            aggregator.process().await;

            assert_eq!(aggregator.summaries.len(), 1);

            let summary = aggregator
                .summaries
                .get::<String>(&"test-service".to_string())
                .unwrap();
            assert_eq!(summary.running.examples.len(), 0);
            assert_eq!(summary.running.count, 0);
            assert_eq!(
                summary
                    .latencies
                    .get(0)
                    .unwrap()
                    .examples
                    .get(0)
                    .unwrap()
                    .span_context
                    .span_id(),
                SpanId::from_u64(12)
            );
            assert_eq!(summary.latencies.get(0).unwrap().count, 1);
        });

        let span = SpanData {
            span_context: SpanContext::new(
                TraceId::from_u128(123),
                SpanId::from_u64(12),
                0,
                true,
                TraceState::default(),
            ),
            name: Cow::from("test-service".to_string()),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            status_code: StatusCode::Ok,
            ..new_test_export_span_data()
        };
        sender.send(TracezMessage::SampleSpan(span.clone())).await?;

        sender.send(TracezMessage::SpanEnd(span.clone())).await?;

        sender.send(TracezMessage::ShutDown).await?;

        handle.await?;

        Ok(())
    }
}
