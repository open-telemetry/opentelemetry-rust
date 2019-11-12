use opentelemetry::{
    api,
    api::Tracer as _,
    api::{Provider, Span},
    sdk,
};
use std::thread;
use std::time::Duration;

fn main() {
    let tracer = sdk::Provider::new().get_tracer("report_example");
    {
        let span0 = tracer.start("main", None);
        thread::sleep(Duration::from_millis(10));
        {
            let mut span1 = tracer.start("sub", Some(span0.get_context()));
            span1.set_attribute(api::Key::new("foo").string("bar"));
            span1.add_event("something wrong".to_string());
            thread::sleep(Duration::from_millis(10));
        }
    }

    // Allow flush
    thread::sleep(Duration::from_millis(250));
}
