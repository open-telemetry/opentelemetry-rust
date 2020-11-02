//! ## zPages processor
//!
//! ZPages processor collect span information when span starts or ends and send it to [`SpanAggregator`]
//! for further process.
//!
//! [`SpanAggregator`]:../struct.SpanAggregator.html
use std::fmt::Formatter;
use std::sync::Mutex;

use futures::channel::mpsc;

use opentelemetry::sdk::trace::{Span, SpanProcessor};
use opentelemetry::{exporter::trace::SpanData, Context};

#[derive(Debug)]
pub enum TracezMessage {
    // Sample span on start
    SampleSpan(SpanData),
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
}

impl SpanProcessor for ZPagesProcessor {
    fn on_start(&self, span: &Span, _cx: &Context) {
        if let Some(data) = span.exported_data() {
            if let Ok(mut inner) = self.0.lock() {
                let _ = inner.channel.try_send(TracezMessage::SampleSpan(data));
            }
        }
    }

    fn on_end(&self, span: SpanData) {
        if let Ok(mut inner) = self.0.lock() {
            let _ = inner.channel.try_send(TracezMessage::SpanEnd(span));
        }
    }

    fn shutdown(&mut self) {
        // do nothing
    }
}
