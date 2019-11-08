use crate::api;
use crate::exporter::trace::jaeger;
use crate::sdk;
use std::thread;

pub struct Tracer(jaeger::Tracer);

impl Tracer {
    pub fn new<S: Into<String>>(service_name: S) -> Self {
        let service_name = service_name.into();
        let (span_tx, span_rx) = crossbeam_channel::bounded(10);
        let tracer = jaeger::Tracer::with_sender(jaeger::AllSampler, span_tx);

        // Spin up thread to report finished spans
        let _ = thread::Builder::new()
            .name("Jaeger span reporter".to_string())
            .spawn(move || {
                let reporter = jaeger::JaegerCompactReporter::new(&service_name)
                    .expect("Can't initialize jaeger reporter");
                for span in span_rx {
                    let _ = reporter.report(&[span]);
                }
            });

        Tracer(tracer)
    }
}

impl api::Tracer for Tracer {
    type Span = sdk::trace::Span;

    fn invalid(&self) -> Self::Span {
        sdk::trace::Span::new(jaeger::Span::inactive())
    }

    fn start(&self, name: String, parent_span: Option<api::SpanContext>) -> Self::Span {
        let start_options = self.0.span(name);
        let started = match parent_span.map(|sc| jaeger::SpanContext::from(sc)) {
            Some(span_context) => start_options.child_of(&span_context).start(),
            None => start_options.start(),
        };

        sdk::trace::Span::new(started)
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
