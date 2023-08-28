//! ## Span Aggregator
//!
//! Process the span information, aggregate counts for latency, running, and errors for spans grouped
//! by name.
use crate::trace::{TracezError, TracezMessage, TracezQuery, TracezResponse};
use crate::SpanQueue;
use async_channel::Receiver;
use futures_util::StreamExt as _;
use opentelemetry::trace::Status;
use opentelemetry_proto::tonic::tracez::v1::TracezCounts;
use opentelemetry_sdk::export::trace::SpanData;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LATENCY_BUCKET: [Duration; 9] = [
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
const LATENCY_BUCKET_COUNT: usize = 9;

/// Aggregate span information from `ZPagesSpanProcessor` and feed that information to server when
/// requested.
#[derive(Debug)]
pub(crate) struct SpanAggregator {
    receiver: Receiver<TracezMessage>,
    summaries: HashMap<String, SpanSummary>,
    sample_size: usize,
}

impl SpanAggregator {
    /// Create a span aggregator
    pub(crate) fn new(receiver: Receiver<TracezMessage>, sample_size: usize) -> SpanAggregator {
        SpanAggregator {
            receiver,
            summaries: HashMap::new(),
            sample_size,
        }
    }

    /// Process request from http server or the span processor.
    pub(crate) async fn process(&mut self) {
        let sample_size = self.sample_size;
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
                                .or_insert_with(|| SpanSummary::new(sample_size));

                            summary.running.remove(span.span_context.clone());

                            if matches!(span.status, Status::Error { .. }) {
                                summary.error.push_back(span);
                            } else {
                                let latency_idx = latency_bucket(span.start_time, span.end_time);
                                if let Some(queue) = summary.latencies.get_mut(latency_idx) {
                                    queue.push_back(span)
                                }
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
                                .or_insert_with(|| SpanSummary::new(sample_size));
                            summary.running.push_back(span)
                        }
                        TracezMessage::Query { query, response_tx } => {
                            let result = self.handle_query(query);
                            let _ = response_tx.send(result);
                        }
                    }
                }
            }
        }
    }

    fn handle_query(&mut self, query: TracezQuery) -> Result<TracezResponse, TracezError> {
        match query {
            TracezQuery::Aggregation => Ok(TracezResponse::Aggregation(
                self.summaries
                    .iter()
                    .map(|(span_name, summary)| TracezCounts {
                        spanname: span_name.clone(),
                        latency: summary
                            .latencies
                            .iter()
                            .map(|queue| queue.count() as u32)
                            .collect(),
                        running: summary.running.count() as u32,
                        error: summary.error.count() as u32,
                    })
                    .collect(),
            )),
            TracezQuery::Latency {
                bucket_index,
                span_name,
            } => self
                .summaries
                .get(&span_name)
                .ok_or(TracezError::NotFound {
                    api: "tracez/api/latency/{bucket_index}/{span_name}",
                })
                .and_then(|summary| {
                    summary
                        .latencies
                        .get(bucket_index)
                        .ok_or(TracezError::InvalidArgument {
                            api: "tracez/api/latency/{bucket_index}/{span_name}",
                            message: "invalid bucket index",
                        })
                        .map(|queue| TracezResponse::Latency(queue.clone().into()))
                }),
            TracezQuery::Error { span_name } => self
                .summaries
                .get(&span_name)
                .ok_or(TracezError::NotFound {
                    api: "tracez/api/error/{span_name}",
                })
                .map(|summary| TracezResponse::Error(summary.error.clone().into())),
            TracezQuery::Running { span_name } => self
                .summaries
                .get(&span_name)
                .ok_or(TracezError::NotFound {
                    api: "tracez/api/error/{span_name}",
                })
                .map(|summary| TracezResponse::Running(summary.running.clone().into())),
        }
    }
}

fn latency_bucket(start_time: SystemTime, end_time: SystemTime) -> usize {
    let latency = end_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        - start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0));
    for (idx, lower) in LATENCY_BUCKET.iter().copied().enumerate().skip(1) {
        if lower > latency {
            return idx - 1;
        }
    }
    LATENCY_BUCKET.len() - 1
}

#[derive(Debug)]
struct SpanSummary {
    running: SpanQueue,
    error: SpanQueue,
    latencies: Vec<SpanQueue>,
}

impl SpanSummary {
    fn new(sample_size: usize) -> SpanSummary {
        SpanSummary {
            running: SpanQueue::new(sample_size),
            error: SpanQueue::new(sample_size),
            latencies: vec![SpanQueue::new(sample_size); LATENCY_BUCKET_COUNT],
        }
    }
}

impl<T: From<SpanData>> From<SpanQueue> for Vec<T> {
    fn from(span_queue: SpanQueue) -> Self {
        span_queue.spans().into_iter().map(Into::into).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::trace::{
        aggregator::{SpanAggregator, LATENCY_BUCKET_COUNT},
        span_queue::SpanQueue,
        TracezMessage,
    };
    use opentelemetry::trace::{SpanContext, SpanId, Status, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::{export::trace::SpanData, testing::trace::new_test_export_span_data};
    use std::borrow::Cow;
    use std::cmp::min;
    use std::time::{Duration, SystemTime};

    enum Action {
        Start,
        End(Duration), // end with latency
    }

    struct ProcessTestPlan {
        // (trace id, span id, trace flag, is error)
        input: Vec<(u128, u64, u8, bool, Action)>,
        // (trace id, span id, trace flag, is error)
        expect_running: Vec<(u128, u64, u8, bool)>,
        // (trace id, span id, trace flag, is error)
        expect_error: Vec<(u128, u64, u8, bool)>,
        // (index of the latency bucket, trace id, span id, trace flag, is error)
        expect_latencies: Vec<(usize, u128, u64, u8, bool)>,
        // name of the test plan
        name: &'static str,
    }

    impl ProcessTestPlan {
        pub(crate) fn get_expect_running(&self) -> Vec<SpanData> {
            self.expect_running
                .iter()
                .cloned()
                .map(|(trace_id, span_id, trace_flag, is_error)| {
                    span_data(trace_id, span_id, trace_flag, is_error)
                })
                .collect()
        }

        pub(crate) fn get_expect_error(&self) -> Vec<SpanData> {
            self.expect_error
                .iter()
                .cloned()
                .map(|(trace_id, span_id, trace_flag, is_error)| {
                    span_data(trace_id, span_id, trace_flag, is_error)
                })
                .collect()
        }

        pub(crate) fn get_latencies(&self) -> Vec<Vec<SpanData>> {
            let mut sink = vec![Vec::new(); LATENCY_BUCKET_COUNT];
            for (index, trace_id, span_id, trace_flag, is_error) in self.expect_latencies.clone() {
                sink.get_mut(index)
                    .unwrap()
                    .push(span_data(trace_id, span_id, trace_flag, is_error))
            }
            sink
        }

        pub(crate) fn get_input(&self) -> (Vec<SpanData>, Vec<SpanData>) {
            let mut start_spans = Vec::new();
            let mut end_spans = Vec::new();
            let start_time = SystemTime::now();
            for input in &self.input {
                let mut span_data = span_data(input.0, input.1, input.2, input.3);
                match input.4 {
                    Action::Start => {
                        span_data.start_time = start_time;
                        start_spans.push(span_data);
                    }
                    Action::End(duration) => {
                        span_data.start_time = start_time;
                        span_data.end_time = start_time.checked_add(duration).unwrap();
                        end_spans.push(span_data);
                    }
                }
            }
            (start_spans, end_spans)
        }
    }

    fn span_data(trace_id: u128, span_id: u64, trace_flag: u8, is_error: bool) -> SpanData {
        let mut span_data = new_test_export_span_data();
        span_data.span_context = SpanContext::new(
            TraceId::from_u128(trace_id),
            SpanId::from_u64(span_id),
            TraceFlags::new(trace_flag),
            true,
            TraceState::default(),
        );
        span_data.name = Cow::from("test-service");
        span_data.status = {
            if is_error {
                Status::error("")
            } else {
                Status::Ok
            }
        };
        span_data
    }

    #[tokio::test]
    async fn test_span_aggregator() -> Result<(), Box<dyn std::error::Error>> {
        const SAMPLE_SIZE: usize = 5;
        let test_cases = vec![
            ProcessTestPlan {
                name: "start and end",
                input: vec![
                    (1, 1, 0, false, Action::Start),
                    (1, 1, 0, false, Action::End(Duration::from_millis(2))),
                ],
                expect_running: vec![],
                expect_error: vec![],
                expect_latencies: vec![(3, 1, 1, 0, false)],
            },
            ProcessTestPlan {
                name: "start and end with error",
                input: vec![
                    (1, 1, 0, false, Action::Start),
                    (1, 1, 0, true, Action::End(Duration::from_millis(2))),
                ],
                expect_latencies: vec![],
                expect_error: vec![(1, 1, 0, true)],
                expect_running: vec![],
            },
            ProcessTestPlan {
                name: "start but not finish",
                input: vec![
                    (1, 2, 0, false, Action::Start),
                    (1, 1, 0, false, Action::Start),
                    (1, 2, 0, false, Action::End(Duration::from_secs(6))),
                ],
                expect_running: vec![(1, 1, 0, false)],
                expect_error: vec![],
                expect_latencies: vec![(6, 1, 2, 0, false)],
            },
            ProcessTestPlan {
                name: "accept spans without started record",
                input: vec![(1, 1, 0, false, Action::End(Duration::from_secs(6)))],
                expect_latencies: vec![(6, 1, 1, 0, false)],
                expect_running: vec![],
                expect_error: vec![],
            },
            ProcessTestPlan {
                name: "evicted spans if the queue is filled",
                input: {
                    let mut input = Vec::with_capacity((SAMPLE_SIZE + 1) * 2);
                    for i in 0..SAMPLE_SIZE + 1 {
                        input.push((1, i as u64 + 1, 0, false, Action::Start));
                        input.push((
                            1,
                            i as u64 + 1,
                            0,
                            false,
                            Action::End(Duration::from_secs(3)),
                        ));
                    }
                    input
                },
                expect_latencies: {
                    let mut latencies = Vec::with_capacity(SAMPLE_SIZE + 1);
                    for i in 0..SAMPLE_SIZE + 1 {
                        latencies.push((6, 1, i as u64 + 1, 0, false));
                    }
                    latencies
                },
                expect_running: vec![],
                expect_error: vec![],
            },
        ];

        let assert_span_queue = |span_queue: &SpanQueue, expected: Vec<SpanData>, msg: String| {
            assert_eq!(span_queue.len(), min(SAMPLE_SIZE, expected.len()));
            for collected_span in span_queue.clone().spans() {
                assert!(
                    expected
                        .iter()
                        .any(|expected_span| collected_span.span_context
                            == expected_span.span_context
                            && collected_span.status == expected_span.status),
                    "{}",
                    msg
                )
            }
        };

        for plan in test_cases {
            let running = plan.get_expect_running();
            let error = plan.get_expect_error();
            let latencies = plan.get_latencies();
            let plan_name = plan.name.to_string();

            let (sender, receiver) = async_channel::unbounded();
            let mut aggregator = SpanAggregator::new(receiver, SAMPLE_SIZE);

            let handle = tokio::spawn(async move {
                aggregator.process().await;

                assert_ne!(aggregator.summaries.len(), 0);
                let summary = aggregator
                    .summaries
                    .get::<String>(&"test-service".to_string())
                    .unwrap();

                assert_span_queue(
                    &summary.running,
                    running,
                    format!(
                        "{} fails because the running status is not expected",
                        plan_name
                    ),
                );
                assert_span_queue(
                    &summary.error,
                    error,
                    format!(
                        "{} fails because the error status is not expected",
                        plan_name
                    ),
                );
                // check the result lengths are expected

                for (index, expected) in (0..LATENCY_BUCKET_COUNT).zip(latencies) {
                    assert_span_queue(
                        summary.latencies.get(index).unwrap(),
                        expected,
                        format!(
                            "{} fails because the latency status with index {} is not expected",
                            plan_name, index,
                        ),
                    );
                }
            });

            let (start_spans, end_spans) = plan.get_input();

            for span in start_spans.into_iter() {
                sender.send(TracezMessage::SampleSpan(span)).await?;
            }

            for span in end_spans.into_iter() {
                sender.send(TracezMessage::SpanEnd(span)).await?;
            }

            sender.send(TracezMessage::ShutDown).await?;

            handle.await?;
        }

        Ok(())
    }
}
