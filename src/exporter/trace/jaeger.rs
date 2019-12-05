//! # OpenTelemetry Jaeger Exporter
//!
//! This exporter currently delegates to the [rustracing_jaeger library]
//! which implements the [OpenTracing API].
//!
//! [rustracing_jaeger library]: https://github.com/sile/rustracing_jaeger
//! [OpenTracing API]: https://opentracing.io/
use crate::exporter::trace;
use crate::{api, sdk};
use std::any;
use std::net;
use std::str::FromStr;
use std::thread;

pub use rustracing::{
    sampler::{AllSampler, NullSampler, PassiveSampler, ProbabilisticSampler},
    tag::{Tag, TagValue},
};
pub use rustracing_jaeger::{
    reporter,
    span::{SpanContext, SpanSender},
    Span, Tracer,
};

/// Default service name no service is configured.
static DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";
static DEFAULT_COLLECTOR_ENDPOINT: &str = "127.0.0.1:6831";

/// Jaeger exporter
#[derive(Debug)]
pub struct Exporter {
    collector_endpoint: net::SocketAddr,
    process: Process,
    pub(crate) span_sender: SpanSender,
}

impl Exporter {
    /// Create a new exporter builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Default `Exporter` with initialized sender.
    pub fn init_default() -> Self {
        Exporter::builder().init()
    }
}

impl trace::SpanExporter for Exporter {
    type Span = sdk::Span;

    /// Ignored because spans export themselves on drop currently.
    fn export(&self, _batch: Vec<Self::Span>) -> Result<trace::ExportResult, ()> {
        Ok(trace::ExportResult::Success)
    }

    /// Ignored for now.
    fn shutdown(&self) {}

    /// Allows `Exporter` to be downcast from trait object.
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}

/// Jaeger process configuration
#[derive(Debug)]
pub struct Process {
    /// The name of the traced service that all spans will be reported as belonging to.
    pub service_name: &'static str,
    /// Metadata about the service that will appear in all `Span`s.
    pub tags: Vec<api::KeyValue>,
}

impl Default for Process {
    /// Default `Process` config
    fn default() -> Self {
        Process {
            service_name: DEFAULT_SERVICE_NAME,
            tags: Default::default(),
        }
    }
}

/// Jaeger Exporter Builder
#[derive(Debug)]
pub struct Builder {
    collector_endpoint: net::SocketAddr,
    process: Process,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            collector_endpoint: DEFAULT_COLLECTOR_ENDPOINT.parse().unwrap(),
            process: Default::default(),
        }
    }
}

impl Builder {
    /// Assign the collector endpoint. Uses `DEFAULT_COLLECTOR_ENDPOINT` if unset.
    pub fn with_collector_endpoint(self, collector_endpoint: net::SocketAddr) -> Self {
        Builder {
            collector_endpoint,
            ..self
        }
    }

    /// Assign the exporter process config.
    pub fn with_process(self, process: Process) -> Self {
        Builder { process, ..self }
    }

    /// Create a new exporter from the builder
    pub fn init(self) -> Exporter {
        let Builder {
            collector_endpoint,
            process: Process { service_name, tags },
        } = self;
        let reporter_tags = tags.clone();
        let (span_tx, span_rx) = crossbeam_channel::bounded(10);

        // Spin up thread to report finished spans
        let _ = thread::Builder::new()
            .name("Jaeger span reporter".to_string())
            .spawn(move || {
                let mut reporter = reporter::JaegerCompactReporter::new(service_name)
                    .expect("Can't initialize jaeger reporter");

                reporter
                    .set_agent_addr(collector_endpoint)
                    .expect("Can't set socket jaeger reporter socket");
                for tag in reporter_tags {
                    let api::KeyValue { key, value } = tag;
                    let value = match value {
                        api::Value::Bool(b) => TagValue::Boolean(b),
                        api::Value::I64(i) => TagValue::Integer(i),
                        api::Value::F64(float) => TagValue::Float(float),
                        v => TagValue::String(v.into()),
                    };
                    reporter.add_service_tag(Tag::new(key, value))
                }
                for span in span_rx {
                    let _ = reporter.report(&[span]);
                }
            });

        Exporter {
            collector_endpoint,
            process: Process { service_name, tags },
            span_sender: span_tx,
        }
    }
}

impl From<api::SpanContext> for rustracing_jaeger::span::SpanContext {
    /// Convert from `api::SpanContext` instances to `rustracing_jaeger`'s `SpanContext` type.
    fn from(context: api::SpanContext) -> Self {
        let jaeger_trace_str = format!(
            "{:x}:{:x}:0:{:x}",
            context.trace_id(),
            context.span_id(),
            context.trace_flags()
        );
        let span_context_state =
            rustracing_jaeger::span::SpanContextState::from_str(&jaeger_trace_str)
                .expect("should always parse");

        rustracing::span::SpanContext::new(span_context_state, Vec::new())
    }
}
