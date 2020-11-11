use crate::{
    exporter::trace::{self as exporter, ExportResult, SpanExporter},
    sdk::{
        trace::{Config, EvictedHashMap, EvictedQueue},
        InstrumentationLibrary,
    },
    trace::{Span, SpanContext, SpanId, SpanKind, StatusCode},
    KeyValue,
};
use async_trait::async_trait;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::SystemTime;
use crate::exporter::trace::SpanData;

#[derive(Debug)]
pub struct TestSpan(pub SpanContext);

impl Span for TestSpan {
    fn add_event_with_timestamp(
        &self,
        _name: String,
        _timestamp: std::time::SystemTime,
        _attributes: Vec<KeyValue>,
    ) {}
    fn span_context(&self) -> &SpanContext {
        &self.0
    }
    fn is_recording(&self) -> bool {
        false
    }
    fn set_attribute(&self, _attribute: KeyValue) {}
    fn set_status(&self, _code: StatusCode, _message: String) {}
    fn update_name(&self, _new_name: String) {}
    fn end_with_timestamp(&self, _timestamp: std::time::SystemTime) {}
}

pub fn new_test_export_span_data() -> exporter::SpanData {
    let config = Config::default();
    exporter::SpanData {
        span_context: SpanContext::empty_context(),
        parent_span_id: SpanId::from_u64(0),
        span_kind: SpanKind::Internal,
        name: "opentelemetry".to_string(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now(),
        attributes: EvictedHashMap::new(config.max_attributes_per_span, 0),
        message_events: EvictedQueue::new(config.max_events_per_span),
        links: EvictedQueue::new(config.max_links_per_span),
        status_code: StatusCode::Unset,
        status_message: "".to_string(),
        resource: config.resource,
        instrumentation_lib: InstrumentationLibrary::default(),
    }
}

#[derive(Debug)]
pub struct TestSpanExporter {
    tx_export: Sender<exporter::SpanData>,
    tx_shutdown: Sender<()>,
}

#[async_trait]
impl SpanExporter for TestSpanExporter {
    async fn export(&mut self, batch: Vec<exporter::SpanData>) -> ExportResult {
        for span_data in batch {
            self.tx_export.send(span_data)?;
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        self.tx_shutdown.send(()).unwrap();
    }
}

pub fn new_test_exporter() -> (TestSpanExporter, Receiver<exporter::SpanData>, Receiver<()>) {
    let (tx_export, rx_export) = channel();
    let (tx_shutdown, rx_shutdown) = channel();
    let exporter = TestSpanExporter {
        tx_export,
        tx_shutdown,
    };
    (exporter, rx_export, rx_shutdown)
}

#[derive(Debug)]
pub struct TokioSpanExporter {
    tx_export: tokio::sync::mpsc::UnboundedSender<exporter::SpanData>,
    tx_shutdown: tokio::sync::mpsc::UnboundedSender<()>,
}

#[async_trait]
impl SpanExporter for TokioSpanExporter {
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        for span_data in batch {
            self.tx_export.send(span_data)?;
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        self.tx_shutdown.send(()).unwrap();
    }
}

pub fn new_tokio_test_exporter() -> (TokioSpanExporter, tokio::sync::mpsc::UnboundedReceiver<exporter::SpanData>, tokio::sync::mpsc::UnboundedReceiver<()>) {
    let (tx_export, rx_export) = tokio::sync::mpsc::unbounded_channel();
    let (tx_shutdown, rx_shutdown) = tokio::sync::mpsc::unbounded_channel();
    let exporter = TokioSpanExporter{
        tx_export,
        tx_shutdown,
    };
    (exporter, rx_export, rx_shutdown)
}

