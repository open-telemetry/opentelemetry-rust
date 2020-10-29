//! ## Span Aggregator
//!
//! Process the span information, aggregate counts for latency, running, and errors for spans grouped
//! by name.
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::channel::mpsc;
use futures::StreamExt;

use opentelemetry::exporter::trace::SpanData;
use opentelemetry::trace::{SpanId, StatusCode};

use crate::trace::span_processor::TracezMessage;

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
pub struct SpanAggregator {
    receiver: mpsc::Receiver<TracezMessage>,
    summaries: HashMap<String, SpanSummary>,
}

impl SpanAggregator {
    /// Create a span aggregator
    pub fn new(receiver: mpsc::Receiver<TracezMessage>) -> SpanAggregator {
        SpanAggregator {
            receiver,
            summaries: HashMap::new(),
        }
    }

    /// Process request from http server or the span processor.
    pub async fn process(&mut self) {
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
                            let summary = self.summaries.entry(span.name.clone()).or_default();
                            summary.running_num -= 1;

                            if summary
                                .running_sample_span
                                .as_ref()
                                .map(|span| span.span_context.span_id())
                                .unwrap_or(SpanId::invalid())
                                == span.span_context.span_id()
                            {
                                // clear current running span if it ended.
                                summary.running_sample_span = None;
                            }
                            if span.status_code != StatusCode::Ok {
                                summary.error_num += 1;
                                summary.error_sample_span = Some(span);
                            } else {
                                let latency_idx = latency_bucket(span.start_time, span.end_time);
                                summary.latency[latency_idx] += 1;
                                summary.latency_sample_span[latency_idx] = Some(span);
                            }
                        }
                        TracezMessage::SampleSpan(span) => {
                            let summary = self.summaries.entry(span.name.clone()).or_default();
                            summary.running_sample_span = Some(span);
                            summary.running_num += 1;
                        }
                        TracezMessage::SpanStart { span_name, .. } => {
                            let summary = self.summaries.entry(span_name).or_default();
                            summary.running_num += 1;
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
    running_sample_span: Option<SpanData>,
    error_sample_span: Option<SpanData>,
    latency_sample_span: [Option<SpanData>; LATENCY_BUCKET_COUNT],

    latency: [usize; 9],
    error_num: usize,
    running_num: usize,
}

impl Default for SpanSummary {
    fn default() -> Self {
        SpanSummary {
            running_sample_span: None,
            error_sample_span: None,
            latency_sample_span: Default::default(),
            latency: Default::default(),
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

    use opentelemetry::exporter::trace::SpanData;
    use opentelemetry::sdk::trace::{EvictedHashMap, EvictedQueue};
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, StatusCode, TraceId, TraceState};

    use crate::trace::span_processor::TracezMessage;
    use crate::trace::SpanAggregator;

    #[tokio::test]
    async fn test_process() -> Result<(), Box<dyn std::error::Error>> {
        let (mut sender, receiver) = mpsc::channel::<TracezMessage>(100);
        let mut aggregator = SpanAggregator::new(receiver);

        let handle = tokio::spawn(async move {
            aggregator.process().await;

            assert_eq!(aggregator.summaries.len(), 1);

            let summary = aggregator
                .summaries
                .get::<String>(&"test-service".to_string())
                .unwrap();
            assert_eq!(summary.running_sample_span, None);
            assert_eq!(summary.running_num, 0);
            assert_eq!(
                summary.latency_sample_span[1]
                    .as_ref()
                    .map(|span| span.span_context.span_id()),
                Some(SpanId::from_u64(12))
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
            name: "test-service".to_string(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: EvictedHashMap::new(12),
            message_events: EvictedQueue::new(12),
            links: EvictedQueue::new(12),
            status_code: StatusCode::Ok,
            status_message: "".to_string(),
            resource: Arc::new(Default::default()),
            instrumentation_lib: Default::default(),
        };
        sender.send(TracezMessage::SampleSpan(span.clone())).await?;

        sender.send(TracezMessage::SpanEnd(span.clone())).await?;

        sender.send(TracezMessage::ShutDown).await?;

        handle.await?;

        Ok(())
    }
}
