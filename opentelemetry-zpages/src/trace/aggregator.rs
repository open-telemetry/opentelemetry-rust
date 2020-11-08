//! ## Span Aggregator
//!
//! Process the span information, aggregate counts for latency, running, and errors for spans grouped
//! by name.
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::channel::mpsc;
use futures::StreamExt;

use opentelemetry::trace::StatusCode;

use crate::trace::TracezMessage;
use crate::SpanQueue;

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
                            summary.running_num = summary.running_num.saturating_sub(1);

                            if span.status_code != StatusCode::Ok {
                                summary.error_num += 1;
                                summary.error_sample_spans.push_back(span.clone());
                            } else {
                                let latency_idx = latency_bucket(span.start_time, span.end_time);
                                summary.latency[latency_idx] += 1;
                                summary.latency_sample_spans[latency_idx].push_back(span.clone());
                            }

                            summary.running_sample_spans.remove(span);
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
                            summary.running_sample_spans.push_back(span);
                            summary.running_num = summary.running_num.saturating_add(1);
                        }
                    }
                }
            }
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
    for idx in 0..LATENCY_BUCKET.len() {
        if LATENCY_BUCKET[idx] > latency {
            return idx as usize;
        }
    }
    return LATENCY_BUCKET.len() - 1;
}

#[derive(Debug)]
struct SpanSummary {
    running_sample_spans: SpanQueue,
    error_sample_spans: SpanQueue,
    latency_sample_spans: [SpanQueue; LATENCY_BUCKET_COUNT],

    latency: [usize; LATENCY_BUCKET_COUNT],
    error_num: usize,
    running_num: usize,
}

impl SpanSummary {
    fn new(sample_size: usize) -> SpanSummary {
        SpanSummary {
            running_sample_spans: SpanQueue::new(sample_size),
            error_sample_spans: SpanQueue::new(sample_size),
            latency_sample_spans: [
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
                SpanQueue::new(sample_size),
            ],
            latency: [0; LATENCY_BUCKET_COUNT],
            error_num: 0,
            running_num: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::SystemTime;

    use futures::channel::mpsc;
    use futures::SinkExt;

    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, StatusCode, TraceId, TraceState};

    use crate::trace::{SpanAggregator, TracezMessage};
    use opentelemetry::sdk::export::trace::SpanData;
    use opentelemetry::sdk::trace::{EvictedHashMap, EvictedQueue};
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
            assert_eq!(summary.running_sample_spans.len(), 0);
            assert_eq!(summary.running_num, 0);
            assert_eq!(
                summary.latency_sample_spans[1]
                    .get(0)
                    .unwrap()
                    .span_context
                    .span_id(),
                SpanId::from_u64(12)
            );
            assert_eq!(summary.latency[1], 1);
        });

        let span = SpanData {
            span_context: SpanContext::new(
                TraceId::from_u128(123),
                SpanId::from_u64(12),
                0,
                true,
                TraceState::default(),
            ),
            parent_span_id: SpanId::invalid(),
            span_kind: SpanKind::Client,
            name: Cow::from("test-service".to_string()),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: EvictedHashMap::new(12, 20),
            events: EvictedQueue::new(12),
            links: EvictedQueue::new(12),
            status_code: StatusCode::Ok,
            status_message: Cow::from("".to_string()),
            resource: Some(Arc::new(Default::default())),
            instrumentation_lib: Default::default(),
        };
        sender.send(TracezMessage::SampleSpan(span.clone())).await?;

        sender.send(TracezMessage::SpanEnd(span.clone())).await?;

        sender.send(TracezMessage::ShutDown).await?;

        handle.await?;

        Ok(())
    }
}
