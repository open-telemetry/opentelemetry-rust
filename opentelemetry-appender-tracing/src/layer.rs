use opentelemetry::logs::{AnyValue, LogRecordBuilder, Logger, LoggerProvider, Severity};
use std::borrow::Cow;
use tracing_subscriber::Layer;

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
struct EventVisitor<'a> {
    log_record_builder: &'a mut LogRecordBuilder,
}

impl<'a> tracing::field::Visit for EventVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.log_record_builder
                .with_body(AnyValue::from(format!("{:?}", value)));
        } else {
            self.log_record_builder
                .with_attribute(field.name(), format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        self.log_record_builder
            .with_attribute(field.name(), value.to_owned());
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        self.log_record_builder.with_attribute(field.name(), value);
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.log_record_builder.with_attribute(field.name(), value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.log_record_builder.with_attribute(field.name(), value);
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
        let mut log_record_builder = LogRecordBuilder::new();
        log_record_builder
            .with_severity_number(map_severity_to_otel_severity(meta.level().as_str()))
            .with_severity_text(meta.level().to_string());

        // Not populating ObservedTimestamp, instead relying on OpenTelemetry
        // API to populate it with current time.

        let mut visitor = EventVisitor {
            log_record_builder: &mut log_record_builder,
        };
        event.record(&mut visitor);
        self.logger.emit(log_record_builder.build());
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(
        &self,
        _event: &tracing_core::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        true
        //let severity = map_severity_to_otel_severity(_event.metadata().level().as_str());
        //self.logger
        //    .event_enabled(severity, _event.metadata().target())
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
