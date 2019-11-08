use crate::api::{SpanContext, Tracer};
use crate::{api, KeyValue};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Debug)]
pub struct BoxedSpan(Box<dyn api::Span>);

impl api::Span for BoxedSpan {
    fn id(&self) -> u64 {
        self.0.id()
    }

    fn parent(&self) -> Option<u64> {
        self.0.parent()
    }

    fn add_event_with_timestamp(&mut self, message: String, timestamp: SystemTime) {
        self.0.add_event_with_timestamp(message, timestamp)
    }

    fn get_context(&self) -> SpanContext {
        self.0.get_context()
    }

    fn is_recording(&self) -> bool {
        self.0.is_recording()
    }

    fn set_attribute(&mut self, attribute: KeyValue) {
        self.0.set_attribute(attribute)
    }

    fn end(&mut self) {
        self.0.end()
    }
}

pub trait BoxedTracer {
    fn invalid_boxed(&self) -> Box<dyn api::Span>;
    fn start_boxed(&self, name: String, parent: Option<api::SpanContext>) -> Box<dyn api::Span>;
}

impl<S: api::Span + 'static> BoxedTracer for Box<dyn api::Tracer<Span = S>> {
    fn invalid_boxed(&self) -> Box<dyn api::Span> {
        Box::new(self.invalid())
    }
    fn start_boxed(&self, name: String, parent: Option<api::SpanContext>) -> Box<dyn api::Span> {
        Box::new(self.start(name, parent))
    }
}

impl Tracer for dyn BoxedTracer {
    type Span = BoxedSpan;

    fn invalid(&self) -> Self::Span {
        BoxedSpan(self.invalid_boxed())
    }

    fn start(&self, name: String, parent_span: Option<api::SpanContext>) -> Self::Span {
        BoxedSpan(self.start_boxed(name, parent_span))
    }

    fn get_active_span(&self) -> Self::Span {
        unimplemented!()
    }

    fn get_span_by_id(&self, _span_id: u64) -> Self::Span {
        unimplemented!()
    }

    fn mark_span_as_active(&self, _span_id: u64) {
        unimplemented!()
    }
}

pub struct GlobalTracer {
    tracer: Box<dyn BoxedTracer>,
}

impl GlobalTracer {
    fn new<S: api::Span + 'static>(tracer: Box<dyn api::Tracer<Span = S>>) -> Self {
        Self {
            tracer: Box::new(tracer),
        }
    }
}

impl api::Tracer for GlobalTracer {
    type Span = BoxedSpan;

    fn invalid(&self) -> Self::Span {
        self.tracer.invalid()
    }

    fn start(&self, name: String, parent_span: Option<api::SpanContext>) -> Self::Span {
        self.tracer.start(name, parent_span)
    }

    fn get_active_span(&self) -> Self::Span {
        unimplemented!()
    }

    fn get_span_by_id(&self, _span_id: u64) -> Self::Span {
        unimplemented!()
    }

    fn mark_span_as_active(&self, _span_id: u64) {
        unimplemented!()
    }
}

thread_local!(static GLOBAL_TRACER: RwLock<Arc<GlobalTracer>> = RwLock::new(Arc::new(GlobalTracer::new(Box::new(api::trace::noop::NoopTracer {})))));

pub fn get_current_tracer() -> Arc<GlobalTracer> {
    GLOBAL_TRACER.with(|tracer_cell| {
        let tracer = tracer_cell.read().unwrap();
        tracer.clone()
    })
}

pub fn set_tracer<S: api::Span + 'static>(new_tracer: Box<dyn api::Tracer<Span = S>>) {
    GLOBAL_TRACER.with(|tracer_cell| {
        let mut tracer = tracer_cell.write().unwrap();
        *tracer = Arc::new(GlobalTracer::new(new_tracer));
    })
}

// Returning noop meter for now
pub fn global_meter() -> crate::api::metrics::noop::NoopMeter {
    crate::api::metrics::noop::NoopMeter {}
}
