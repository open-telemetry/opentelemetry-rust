use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
use std::borrow::Cow;
use tracing_core::{Level, Subscriber};
use tracing_subscriber::{registry::LookupSpan, Layer};

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
struct EventVisitor<'a> {
    log_record: &'a mut LogRecord,
}

impl<'a> tracing::field::Visit for EventVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.log_record.body = Some(format!("{value:?}").into());
        } else if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), format!("{value:?}").into()));
        } else {
            let vec = vec![(field.name().into(), format!("{value:?}").into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.to_owned().into()));
        } else {
            let vec = vec![(field.name().into(), value.to_owned().into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
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
    S: Subscriber + for<'a> LookupSpan<'a>,
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
        log_record.severity_number = Some(severity_of_level(meta.level()));
        log_record.severity_text = Some(meta.level().to_string().into());

        // Extract the trace_id & span_id from the opentelemetry extension.
        #[cfg(feature = "tracing")]
        inject_trace_context(&mut log_record, &_ctx);

        // add the `name` metadata to attributes
        // TBD - Propose this to be part of log_record metadata.
        let vec = vec![("name", meta.name())];
        log_record.attributes = Some(vec.into_iter().map(|(k, v)| (k.into(), v.into())).collect());

        // Not populating ObservedTimestamp, instead relying on OpenTelemetry
        // API to populate it with current time.

        let mut visitor = EventVisitor {
            log_record: &mut log_record,
        };
        event.record(&mut visitor);
        self.logger.emit(log_record);
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(
        &self,
        _event: &tracing_core::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let severity = severity_of_level(_event.metadata().level());
        self.logger
            .event_enabled(severity, _event.metadata().target())
    }
}

#[cfg(feature = "tracing")]
fn inject_trace_context<S>(
    log_record: &mut LogRecord,
    ctx: &tracing_subscriber::layer::Context<'_, S>,
) where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use opentelemetry::{
        logs::TraceContext,
        trace::{SpanContext, TraceFlags, TraceState},
    };

    if let Some((trace_id, span_id)) = ctx.lookup_current().and_then(|span| {
        span.extensions()
            .get::<tracing_opentelemetry::OtelData>()
            .and_then(|ext| ext.builder.trace_id.zip(ext.builder.span_id))
    }) {
        log_record.trace_context = Some(TraceContext::from(&SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::default(),
            false,
            TraceState::default(),
        )));
    }
}

const fn severity_of_level(level: &Level) -> Severity {
    match *level {
        Level::TRACE => Severity::Trace,
        Level::DEBUG => Severity::Debug,
        Level::INFO => Severity::Info,
        Level::WARN => Severity::Warn,
        Level::ERROR => Severity::Error,
    }
}

#[cfg(test)]
mod tests {
    use crate::layer;
    use opentelemetry::logs::Severity;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::trace::{TraceContextExt, TraceFlags, Tracer};
    use opentelemetry::{logs::AnyValue, Key};
    use opentelemetry_sdk::logs::LoggerProvider;
    use opentelemetry_sdk::testing::logs::InMemoryLogsExporter;
    use opentelemetry_sdk::trace::{config, Sampler, TracerProvider};
    use tracing::error;
    use tracing_subscriber::layer::SubscriberExt;

    // cargo test --features=testing
    #[test]
    fn tracing_appender_standalone() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);

        // Act
        error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
        logger_provider.force_flush();

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .get(0)
            .expect("Atleast one log is expected to be present.");

        // Validate common fields
        assert_eq!(log.instrumentation.name, "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number, Some(Severity::Error));

        // Validate trace context is none.
        assert!(log.record.trace_context.is_none());

        // Validate attributes
        let attributes: Vec<(Key, AnyValue)> = log
            .record
            .attributes
            .clone()
            .expect("Attributes are expected");
        assert_eq!(attributes.len(), 4);
        assert!(attributes.contains(&(Key::new("name"), "my-event-name".into())));
        assert!(attributes.contains(&(Key::new("event_id"), 20.into())));
        assert!(attributes.contains(&(Key::new("user_name"), "otel".into())));
        assert!(attributes.contains(&(Key::new("user_email"), "otel@opentelemetry.io".into())));
    }

    #[test]
    fn tracing_appender_inside_tracing_context() {
        // Arrange
        let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
        let logger_provider = LoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);

        // setup tracing as well.
        let tracer_provider = TracerProvider::builder()
            .with_config(config().with_sampler(Sampler::AlwaysOn))
            .build();
        let tracer = tracer_provider.tracer("test-tracer");

        // Act
        let (trace_id_expected, span_id_expected) = tracer.in_span("test-span", |cx| {
            let trace_id = cx.span().span_context().trace_id();
            let span_id = cx.span().span_context().span_id();

            // logging is done inside span context.
            error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
           (trace_id, span_id)
        });

        logger_provider.force_flush();

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .get(0)
            .expect("Atleast one log is expected to be present.");

        // validate common fields.
        assert_eq!(log.instrumentation.name, "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number, Some(Severity::Error));

        // validate trace context.
        assert!(log.record.trace_context.is_some());
        assert_eq!(
            log.record.trace_context.as_ref().unwrap().trace_id,
            trace_id_expected
        );
        assert_eq!(
            log.record.trace_context.as_ref().unwrap().span_id,
            span_id_expected
        );
        assert_eq!(
            log.record
                .trace_context
                .as_ref()
                .unwrap()
                .trace_flags
                .unwrap(),
            TraceFlags::SAMPLED
        );

        // validate attributes.
        let attributes: Vec<(Key, AnyValue)> = log
            .record
            .attributes
            .clone()
            .expect("Attributes are expected");
        assert_eq!(attributes.len(), 4);
        assert!(attributes.contains(&(Key::new("name"), "my-event-name".into())));
        assert!(attributes.contains(&(Key::new("event_id"), 20.into())));
        assert!(attributes.contains(&(Key::new("user_name"), "otel".into())));
        assert!(attributes.contains(&(Key::new("user_email"), "otel@opentelemetry.io".into())));
    }
}
