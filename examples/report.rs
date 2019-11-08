use opentelemetry::{api::Span, api::Tracer as _, sdk};
use std::thread;
use std::time::Duration;

fn main() {
    let tracer = sdk::Tracer::new("report_example");
    {
        let parent: Option<opentelemetry::api::SpanContext> = None;
        let span0 = tracer.start("main".to_string(), parent);
        thread::sleep(Duration::from_millis(10));
        {
            let mut span1 = tracer.start("sub".to_string(), Some(span0.get_context()));
            span1.set_attribute(opentelemetry::Key::new("foo").string("bar"));
            span1.add_event("something wrong".to_string());
            thread::sleep(Duration::from_millis(10));
        }
    }

    // Allow flush
    thread::sleep(Duration::from_millis(250));
}
