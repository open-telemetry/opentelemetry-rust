//! ## zPages processor
//!
//! ZPages processor collect span information when span starts or ends and send it to [`SpanAggregator`]
//! for further process.
//!
//! [`SpanAggregator`]:../struct.SpanAggregator.html
use std::fmt::Formatter;

use async_channel::Sender;

use crate::trace::TracezMessage;
use opentelemetry::sdk::trace::{Span, SpanProcessor};
use opentelemetry::trace::TraceResult;
use opentelemetry::{sdk::export::trace::SpanData, Context};

/// ZPagesProcessor is an alternative to external exporters. It sends span data to zPages server
/// where it will be archive and user can use this information for debug purpose.
///
/// When span starts, the zPages processor determine whether there is a span with the span name
/// is sampled already. If not, send the whole span for sampling purpose. Otherwise, just send span
/// name, span id to avoid the unnecessary clone [`SpanAggregator`] .
/// When span ends, the zPages processor will send complete span data to [`SpanAggregator`] .
///
pub struct ZPagesSpanProcessor {
    tx: Sender<TracezMessage>,
}

impl std::fmt::Debug for ZPagesSpanProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ZPageProcessor")
    }
}

impl ZPagesSpanProcessor {
    /// Create a new `ZPagesSpanProcessor`.
    pub fn new(tx: Sender<TracezMessage>) -> ZPagesSpanProcessor {
        ZPagesSpanProcessor { tx }
    }
}

impl SpanProcessor for ZPagesSpanProcessor {
    fn on_start(&self, span: &Span, _cx: &Context) {
        // if the aggregator is already dropped. This is a no-op
        if let Some(data) = span.exported_data() {
            let _ = self.tx.try_send(TracezMessage::SampleSpan(data));
        }
    }

    fn on_end(&self, span: SpanData) {
        // if the aggregator is already dropped. This is a no-op
        let _ = self.tx.try_send(TracezMessage::SpanEnd(span));
    }

    fn force_flush(&self) -> TraceResult<()> {
        // do nothing
        Ok(())
    }

    fn shutdown(&mut self) -> TraceResult<()> {
        // do nothing
        Ok(())
    }
}
