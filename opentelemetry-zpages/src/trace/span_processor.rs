//! ## zPages processor
//!
//! ZPages processor collect span information when span starts or ends and send it to [`SpanAggregator`]
//! for further process.
//!
//! [`SpanAggregator`]:../struct.SpanAggregator.html
use futures::channel::mpsc;
use opentelemetry::{
    exporter::trace::SpanData,
    trace::{SpanId, SpanProcessor},
};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Mutex;

#[derive(Debug)]
pub enum TracezMessage {
    // Sample span on start
    SampleSpan(SpanData),
    // Send span information on start
    SpanStart { span_name: String, span_id: SpanId },
    SpanEnd(SpanData),
    ShutDown,
}

/// ZPagesProcessor is an alternative to external exporters. It sends span data to zPages server
/// where it will be archive and user can use this information for debug purpose.
///
/// When span starts, the zPages processor determine whether there is a span with the span name is sampled already. If not, send the whole span for sampling purpose. Otherwise, just send span name, span id to avoid the unnecessary clone [`SpanAggregator`] .
/// When span ends, the zPages processor will send complete span data to [`SpanAggregator`] .
///
pub struct ZPagesProcessor(Mutex<Inner>);

impl std::fmt::Debug for ZPagesProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ZPageProcessor")
    }
}

struct Inner {
    channel: mpsc::Sender<TracezMessage>,
    current_sampled_span: HashMap<String, SpanId>,
}

impl SpanProcessor for ZPagesProcessor {
    fn on_start(&self, span: &SpanData) {
        if let Ok(mut inner) = self.0.lock() {
            // if we already have span sampled for this span name, just send span name, span id.
            // if not, then send the whole span for sampling.
            if inner.current_sampled_span.contains_key(&span.name) {
                let _ = inner.channel.try_send(TracezMessage::SpanStart {
                    span_name: span.name.clone(),
                    span_id: span.span_context.span_id(),
                });
            } else {
                let _ = inner
                    .channel
                    .try_send(TracezMessage::SampleSpan(span.clone()))
                    .and_then(|_| {
                        inner
                            .current_sampled_span
                            .insert(span.name.clone(), span.span_context.span_id());
                        Ok(())
                    });
            }
        }
    }

    fn on_end(&self, span: SpanData) {
        if let Ok(mut inner) = self.0.lock() {
            // if the sampled span ends, remove it from map so that next span can be sampled.
            if inner.current_sampled_span.contains_key(&span.name) {
                inner.current_sampled_span.remove(&span.name);
            }
            let _ = inner.channel.try_send(TracezMessage::SpanEnd(span));
        }
    }

    fn shutdown(&mut self) {
        // do nothing
    }
}
