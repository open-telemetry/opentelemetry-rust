pub use in_memory_exporter::{InMemorySpanExporter, InMemorySpanExporterBuilder};

mod in_memory_exporter;

use crate::{
    export::{
        trace::{ExportResult, SpanData, SpanExporter},
        ExportError,
    },
    trace::{Config, EvictedQueue, Links},
    InstrumentationLibrary,
};
use async_trait::async_trait;
use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use futures_util::future::BoxFuture;
pub use opentelemetry::testing::trace::TestSpan;
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use std::fmt::{Display, Formatter};

pub fn new_test_export_span_data() -> SpanData {
    let config = Config::default();
    SpanData {
        span_context: SpanContext::new(
            TraceId::from_u128(1),
            SpanId::from_u64(1),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        ),
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: "opentelemetry".into(),
        start_time: opentelemetry::time::now(),
        end_time: opentelemetry::time::now(),
        attributes: Vec::new(),
        dropped_attributes_count: 0,
        events: EvictedQueue::new(config.span_limits.max_events_per_span),
        links: Links::default(),
        status: Status::Unset,
        resource: config.resource,
        instrumentation_lib: InstrumentationLibrary::default(),
    }
}

#[derive(Debug)]
pub struct TestSpanExporter {
    tx_export: Sender<SpanData>,
    tx_shutdown: Sender<()>,
}

#[async_trait]
impl SpanExporter for TestSpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        for span_data in batch {
            if let Err(err) = self
                .tx_export
                .send(span_data)
                .map_err::<TestExportError, _>(Into::into)
            {
                return Box::pin(std::future::ready(Err(Into::into(err))));
            }
        }
        Box::pin(std::future::ready(Ok(())))
    }

    fn shutdown(&mut self) {
        let _ = self.tx_shutdown.send(()); // ignore error
    }
}

pub fn new_test_exporter() -> (TestSpanExporter, Receiver<SpanData>, Receiver<()>) {
    let (tx_export, rx_export) = unbounded();
    let (tx_shutdown, rx_shutdown) = unbounded();
    let exporter = TestSpanExporter {
        tx_export,
        tx_shutdown,
    };
    (exporter, rx_export, rx_shutdown)
}

#[derive(Debug)]
pub struct TokioSpanExporter {
    tx_export: tokio::sync::mpsc::UnboundedSender<SpanData>,
    tx_shutdown: tokio::sync::mpsc::UnboundedSender<()>,
}

impl SpanExporter for TokioSpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        for span_data in batch {
            if let Err(err) = self
                .tx_export
                .send(span_data)
                .map_err::<TestExportError, _>(Into::into)
            {
                return Box::pin(std::future::ready(Err(Into::into(err))));
            }
        }
        Box::pin(std::future::ready(Ok(())))
    }

    fn shutdown(&mut self) {
        self.tx_shutdown.send(()).unwrap();
    }
}

pub fn new_tokio_test_exporter() -> (
    TokioSpanExporter,
    tokio::sync::mpsc::UnboundedReceiver<SpanData>,
    tokio::sync::mpsc::UnboundedReceiver<()>,
) {
    let (tx_export, rx_export) = tokio::sync::mpsc::unbounded_channel();
    let (tx_shutdown, rx_shutdown) = tokio::sync::mpsc::unbounded_channel();
    let exporter = TokioSpanExporter {
        tx_export,
        tx_shutdown,
    };
    (exporter, rx_export, rx_shutdown)
}

#[derive(Debug)]
pub struct TestExportError(String);

impl std::error::Error for TestExportError {}

impl ExportError for TestExportError {
    fn exporter_name(&self) -> &'static str {
        "test"
    }
}

impl Display for TestExportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for TestExportError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        TestExportError(err.to_string())
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for TestExportError {
    fn from(err: SendError<T>) -> Self {
        TestExportError(err.to_string())
    }
}

/// A no-op instance of an [`SpanExporter`].
///
/// [`SpanExporter`]: crate::export::trace::SpanExporter
#[derive(Debug, Default)]
pub struct NoopSpanExporter {
    _private: (),
}

impl NoopSpanExporter {
    /// Create a new noop span exporter
    pub fn new() -> Self {
        NoopSpanExporter { _private: () }
    }
}

#[async_trait::async_trait]
impl SpanExporter for NoopSpanExporter {
    fn export(&mut self, _: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        Box::pin(std::future::ready(Ok(())))
    }
}
