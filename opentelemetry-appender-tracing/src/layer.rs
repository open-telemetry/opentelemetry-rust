use opentelemetry_api::{
    logs::LoggerProvider,
    logs::Severity,
    logs::{LogRecord, Logger},
    OrderMap,
};

use tracing_subscriber::Layer;

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
struct EventVisitor<'a> {
    log_record: &'a mut LogRecord,
}

impl<'a> tracing::field::Visit for EventVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.log_record.body = Some(format!("{value:?}").into());
        } else if let Some(ref mut map) = self.log_record.attributes {
            map.insert(field.name().into(), format!("{value:?}").into());
        } else {
            let mut map = OrderMap::with_capacity(1);
            map.insert(field.name().into(), format!("{value:?}").into());
            self.log_record.attributes = Some(map);
        }
    }

    // TODO: All record functions should be implemented instead of relying
    // of record_debug which stores everything as string.
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
            logger: provider.logger(INSTRUMENTATION_LIBRARY_NAME),
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

        let mut visitor = EventVisitor {
            log_record: &mut log_record,
        };
        event.record(&mut visitor);
        self.logger.emit(log_record);
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
