use crate::api;
use crate::exporter::trace::jaeger;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Span(jaeger::Span);

impl Span {
    pub(crate) fn new(span: jaeger::Span) -> Self {
        Span(span)
    }
}

impl api::Span for Span {
    fn id(&self) -> u64 {
        self.get_context().span_id()
    }

    fn parent(&self) -> Option<u64> {
        // TODO
        None
    }

    fn add_event_with_timestamp(&mut self, message: String, _timestamp: SystemTime) {
        self.0.log(|log| {
            log.std().message(message);
        });
    }

    fn get_context(&self) -> api::SpanContext {
        match self.0.context() {
            Some(context) => {
                let state = context.state();
                let trace_id = u128::from_str_radix(&state.trace_id().to_string(), 16).unwrap();
                let trace_flags = if state.is_sampled() { 1 } else { 0 };

                api::SpanContext::new(trace_id, state.span_id(), trace_flags)
            }
            None => api::SpanContext::new(rand::random(), 0, 0),
        }
    }

    fn is_recording(&self) -> bool {
        true
    }

    fn set_attribute(&mut self, attribute: crate::KeyValue) {
        let crate::KeyValue { key, value } = attribute;
        self.0.set_tag(|| jaeger::Tag::new(key, value.to_string()))
    }

    fn end(&mut self) {
        self.0.set_finish_time(SystemTime::now)
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        println!("DROPPING SPAN!");
    }
}
