use crate::api::{
    trace::{Span, SpanReference, StatusCode},
    KeyValue,
};

#[derive(Debug)]
pub struct TestSpan(pub SpanReference);

impl Span for TestSpan {
    fn add_event_with_timestamp(
        &self,
        _name: String,
        _timestamp: std::time::SystemTime,
        _attributes: Vec<KeyValue>,
    ) {
    }
    fn span_reference(&self) -> SpanReference {
        self.0.clone()
    }
    fn is_recording(&self) -> bool {
        false
    }
    fn set_attribute(&self, _attribute: KeyValue) {}
    fn set_status(&self, _code: StatusCode, _message: String) {}
    fn update_name(&self, _new_name: String) {}
    fn end_with_timestamp(&self, _timestamp: std::time::SystemTime) {}
}
