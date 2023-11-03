use opentelemetry::{
    logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity},
    Key,
};
use std::borrow::Cow;
use tracing_core::Metadata;
use tracing_subscriber::Layer;

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
#[derive(Default)]
struct EventVisitor {
    log_record_attributes: Vec<(Key, AnyValue)>,
    log_record_body: Option<AnyValue>,
}

impl<'a> EventVisitor {
    fn visit_metadata(&mut self, meta: &'static Metadata<'static>) {
        self.log_record_attributes
            .push(("log.name".into(), meta.name().into()));
        self.log_record_attributes
            .push(("log.target".into(), meta.target().into()));

        if let Some(module_path) = meta.module_path() {
            self.log_record_attributes
                .push(("log.module.path".into(), module_path.into()));
        }
        if let Some(file) = meta.file() {
            self.log_record_attributes
                .push(("log.file.name".into(), file.into()));
        }
        if let Some(line) = meta.line() {
            self.log_record_attributes
                .push(("log.file.line".into(), line.into()));
        }
    }

    fn push_to_otel_log_record(self, log_record: &mut LogRecord) {
        log_record.body = self.log_record_body;
        log_record.attributes = Some(self.log_record_attributes);
    }
}

impl tracing::field::Visit for EventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.log_record_body = Some(format!("{value:?}").into());
        } else {
            self.log_record_attributes
                .push((field.name().into(), format!("{value:?}").into()));
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        self.log_record_attributes
            .push((field.name().into(), value.to_owned().into()));
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        self.log_record_attributes
            .push((field.name().into(), value.into()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.log_record_attributes
            .push((field.name().into(), value.into()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.log_record_attributes
            .push((field.name().into(), value.into()));
    }

    // TODO: Remaining field types from AnyValue : Bytes, ListAny, Boolean
}

pub struct OpenTelemetryTracingBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    logger: L,
    _phantom: std::marker::PhantomData<P>, // P is not used.
}

impl<P, L> OpenTelemetryTracingBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    pub fn new(provider: &P) -> Self {
        OpenTelemetryTracingBridge {
            logger: provider.versioned_logger(
                INSTRUMENTATION_LIBRARY_NAME,
                Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
                None,
                None,
            ),
            _phantom: Default::default(),
        }
    }
}

impl<S, P, L> Layer<S> for OpenTelemetryTracingBridge<P, L>
where
    S: tracing::Subscriber,
    P: LoggerProvider<Logger = L> + Send + Sync + 'static,
    L: Logger + Send + Sync + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let mut log_record: LogRecord = LogRecord::default();
        log_record.severity_number = Some(map_severity_to_otel_severity(meta.level().as_str()));
        log_record.severity_text = Some(meta.level().to_string().into());

        // Not populating ObservedTimestamp, instead relying on OpenTelemetry
        // API to populate it with current time.

        let mut visitor = EventVisitor::default();
        visitor.visit_metadata(meta);
        // Visit fields.
        event.record(&mut visitor);
        visitor.push_to_otel_log_record(&mut log_record);

        self.logger.emit(log_record);
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(
        &self,
        _event: &tracing_core::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let severity = map_severity_to_otel_severity(_event.metadata().level().as_str());
        self.logger
            .event_enabled(severity, _event.metadata().target())
    }
}

fn map_severity_to_otel_severity(level: &str) -> Severity {
    match level {
        "INFO" => Severity::Info,
        "DEBUG" => Severity::Debug,
        "TRACE" => Severity::Trace,
        "WARN" => Severity::Warn,
        "ERROR" => Severity::Error,
        _ => Severity::Info, // won't reach here
    }
}
