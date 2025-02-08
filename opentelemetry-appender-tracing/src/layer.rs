use opentelemetry::{
    logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity},
    InstrumentationScope, Key,
};
use std::borrow::Cow;
use tracing_core::Level;
#[cfg(feature = "experimental_metadata_attributes")]
use tracing_core::Metadata;
#[cfg(feature = "experimental_metadata_attributes")]
use tracing_log::NormalizeEvent;
use tracing_subscriber::{registry::LookupSpan, Layer};

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
struct EventVisitor<'a, LR: LogRecord> {
    log_record: &'a mut LR,
}

/// Logs from the log crate have duplicated attributes that we removed here.
#[cfg(feature = "experimental_metadata_attributes")]
fn is_duplicated_metadata(field: &'static str) -> bool {
    field
        .strip_prefix("log.")
        .map(|remainder| matches!(remainder, "file" | "line" | "module_path" | "target"))
        .unwrap_or(false)
}

#[cfg(feature = "experimental_metadata_attributes")]
fn get_filename(filepath: &str) -> &str {
    if let Some((_, filename)) = filepath.rsplit_once('/') {
        return filename;
    }
    if let Some((_, filename)) = filepath.rsplit_once('\\') {
        return filename;
    }
    filepath
}

impl<'a, LR: LogRecord> EventVisitor<'a, LR> {
    fn new(log_record: &'a mut LR) -> Self {
        EventVisitor { log_record }
    }

    #[cfg(feature = "experimental_metadata_attributes")]
    fn visit_experimental_metadata(&mut self, meta: &Metadata) {
        if let Some(module_path) = meta.module_path() {
            self.log_record.add_attribute(
                Key::new("code.namespace"),
                AnyValue::from(module_path.to_owned()),
            );
        }

        if let Some(filepath) = meta.file() {
            self.log_record.add_attribute(
                Key::new("code.filepath"),
                AnyValue::from(filepath.to_owned()),
            );
            self.log_record.add_attribute(
                Key::new("code.filename"),
                AnyValue::from(get_filename(filepath).to_owned()),
            );
        }

        if let Some(line) = meta.line() {
            self.log_record
                .add_attribute(Key::new("code.lineno"), AnyValue::from(line));
        }
    }
}

impl<LR: LogRecord> tracing::field::Visit for EventVisitor<'_, LR> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        #[cfg(feature = "experimental_metadata_attributes")]
        if is_duplicated_metadata(field.name()) {
            return;
        }
        if field.name() == "message" {
            self.log_record.set_body(format!("{:?}", value).into());
        } else {
            self.log_record
                .add_attribute(Key::new(field.name()), AnyValue::from(format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        #[cfg(feature = "experimental_metadata_attributes")]
        if is_duplicated_metadata(field.name()) {
            return;
        }
        //TODO: Consider special casing "message" to populate body and document
        // to users to use message field for log message, to avoid going to the
        // record_debug, which has dyn dispatch, string allocation and
        // formatting cost.

        //TODO: Fix heap allocation. Check if lifetime of &str can be used
        // to optimize sync exporter scenario.
        self.log_record
            .add_attribute(Key::new(field.name()), AnyValue::from(value.to_owned()));
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        self.log_record
            .add_attribute(Key::new(field.name()), AnyValue::from(value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.log_record
            .add_attribute(Key::new(field.name()), AnyValue::from(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        #[cfg(feature = "experimental_metadata_attributes")]
        if is_duplicated_metadata(field.name()) {
            return;
        }
        self.log_record
            .add_attribute(Key::new(field.name()), AnyValue::from(value));
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
        let scope = InstrumentationScope::builder(INSTRUMENTATION_LIBRARY_NAME)
            .with_version(Cow::Borrowed(env!("CARGO_PKG_VERSION")))
            .build();

        OpenTelemetryTracingBridge {
            logger: provider.logger_with_scope(scope),
            _phantom: Default::default(),
        }
    }
}

impl<S, P, L> Layer<S> for OpenTelemetryTracingBridge<P, L>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    P: LoggerProvider<Logger = L> + Send + Sync + 'static,
    L: Logger + Send + Sync + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        #[cfg(feature = "experimental_metadata_attributes")]
        let normalized_meta = event.normalized_metadata();

        #[cfg(feature = "experimental_metadata_attributes")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        #[cfg(not(feature = "experimental_metadata_attributes"))]
        let meta = event.metadata();

        let mut log_record = self.logger.create_log_record();

        // TODO: Fix heap allocation
        log_record.set_target(meta.target().to_string());
        log_record.set_event_name(meta.name());
        log_record.set_severity_number(severity_of_level(meta.level()));
        log_record.set_severity_text(meta.level().as_str());
        let mut visitor = EventVisitor::new(&mut log_record);
        #[cfg(feature = "experimental_metadata_attributes")]
        visitor.visit_experimental_metadata(meta);
        // Visit fields.
        event.record(&mut visitor);

        /*#[cfg(feature = "experimental_use_tracing_span_context")]
        if let Some(span) = _ctx.event_span(event) {
            use tracing_opentelemetry::OtelData;
            let opt_span_id = span
                .extensions()
                .get::<OtelData>()
                .and_then(|otd| otd.builder.span_id);

            let opt_trace_id = span.scope().last().and_then(|root_span| {
                root_span
                    .extensions()
                    .get::<OtelData>()
                    .and_then(|otd| otd.builder.trace_id)
            });

            if let Some((trace_id, span_id)) = opt_trace_id.zip(opt_span_id) {
                log_record.set_trace_context(trace_id, span_id, None);
            }
        } */

        //emit record
        self.logger.emit(log_record);
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
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
    use opentelemetry::trace::TracerProvider;
    use opentelemetry::trace::{TraceContextExt, TraceFlags, Tracer};
    use opentelemetry::{logs::AnyValue, Key};
    use opentelemetry_sdk::error::OTelSdkResult;
    use opentelemetry_sdk::logs::InMemoryLogExporter;
    use opentelemetry_sdk::logs::{LogBatch, LogExporter};
    use opentelemetry_sdk::logs::{SdkLogRecord, SdkLoggerProvider};
    use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
    use tracing::{error, warn};
    use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer};

    pub fn attributes_contains(log_record: &SdkLogRecord, key: &Key, value: &AnyValue) -> bool {
        log_record
            .attributes_iter()
            .any(|(k, v)| k == key && v == value)
    }

    fn create_tracing_subscriber(
        _exporter: InMemoryLogExporter,
        logger_provider: &SdkLoggerProvider,
    ) -> impl tracing::Subscriber {
        let level_filter = tracing_subscriber::filter::LevelFilter::WARN; // Capture WARN and ERROR levels
        let layer =
            layer::OpenTelemetryTracingBridge::new(logger_provider).with_filter(level_filter); // No filter based on target, only based on log level

        tracing_subscriber::registry().with(layer)
    }

    // cargo test --features=testing

    #[derive(Clone, Debug, Default)]
    struct ReentrantLogExporter;

    impl LogExporter for ReentrantLogExporter {
        #[allow(clippy::manual_async_fn)]
        fn export(
            &self,
            _batch: LogBatch<'_>,
        ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
            async {
                // This will cause a deadlock as the export itself creates a log
                // while still within the lock of the SimpleLogProcessor.
                warn!(name: "my-event-name", target: "reentrant", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
                Ok(())
            }
        }
    }

    #[test]
    #[ignore = "See issue: https://github.com/open-telemetry/opentelemetry-rust/issues/1745"]
    fn simple_processor_deadlock() {
        let exporter: ReentrantLogExporter = ReentrantLogExporter;
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);

        // Setting subscriber as global as that is the only way to test this scenario.
        tracing_subscriber::registry().with(layer).init();
        warn!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
    }

    #[test]
    #[ignore = "While this test runs fine, this uses global subscriber and does not play well with other tests."]
    fn simple_processor_no_deadlock() {
        let exporter: ReentrantLogExporter = ReentrantLogExporter;
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);

        // This filter will prevent the deadlock as the reentrant log will be
        // ignored.
        let filter = EnvFilter::new("debug").add_directive("reentrant=error".parse().unwrap());
        // Setting subscriber as global as that is the only way to test this scenario.
        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
        warn!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore = "While this test runs fine, this uses global subscriber and does not play well with other tests."]
    async fn batch_processor_no_deadlock() {
        let exporter: ReentrantLogExporter = ReentrantLogExporter;
        let logger_provider = SdkLoggerProvider::builder()
            .with_batch_exporter(exporter.clone())
            .build();

        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);

        tracing_subscriber::registry().with(layer).init();
        warn!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
    }

    #[test]
    fn tracing_appender_standalone() {
        // Arrange
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let subscriber = create_tracing_subscriber(exporter.clone(), &logger_provider);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);

        // Act
        error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
        assert!(logger_provider.force_flush().is_ok());

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .first()
            .expect("Atleast one log is expected to be present.");

        // Validate common fields
        assert_eq!(log.instrumentation.name(), "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number(), Some(Severity::Error));

        // Validate trace context is none.
        assert!(log.record.trace_context().is_none());

        // Validate attributes
        #[cfg(not(feature = "experimental_metadata_attributes"))]
        assert_eq!(log.record.attributes_iter().count(), 3);
        #[cfg(feature = "experimental_metadata_attributes")]
        assert_eq!(log.record.attributes_iter().count(), 7);
        assert!(attributes_contains(
            &log.record,
            &Key::new("event_id"),
            &AnyValue::Int(20)
        ));
        assert!(attributes_contains(
            &log.record,
            &Key::new("user_name"),
            &AnyValue::String("otel".into())
        ));
        assert!(attributes_contains(
            &log.record,
            &Key::new("user_email"),
            &AnyValue::String("otel@opentelemetry.io".into())
        ));
        #[cfg(feature = "experimental_metadata_attributes")]
        {
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.filename"),
                &AnyValue::String("layer.rs".into())
            ));
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.namespace"),
                &AnyValue::String("opentelemetry_appender_tracing::layer::tests".into())
            ));
            // The other 3 experimental_metadata_attributes are too unstable to check their value.
            // Ex.: The path will be different on a Windows and Linux machine.
            // Ex.: The line can change easily if someone makes changes in this source file.
            let attributes_key: Vec<Key> = log
                .record
                .attributes_iter()
                .map(|(key, _)| key.clone())
                .collect();
            assert!(attributes_key.contains(&Key::new("code.filepath")));
            assert!(attributes_key.contains(&Key::new("code.lineno")));
            assert!(!attributes_key.contains(&Key::new("log.target")));
        }
    }

    #[test]
    fn tracing_appender_inside_tracing_context() {
        // Arrange
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let subscriber = create_tracing_subscriber(exporter.clone(), &logger_provider);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);

        // setup tracing as well.
        let tracer_provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
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

        assert!(logger_provider.force_flush().is_ok());

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .first()
            .expect("Atleast one log is expected to be present.");

        // validate common fields.
        assert_eq!(log.instrumentation.name(), "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number(), Some(Severity::Error));

        // validate trace context.
        assert!(log.record.trace_context().is_some());
        assert_eq!(
            log.record.trace_context().unwrap().trace_id,
            trace_id_expected
        );
        assert_eq!(
            log.record.trace_context().unwrap().span_id,
            span_id_expected
        );
        assert_eq!(
            log.record.trace_context().unwrap().trace_flags.unwrap(),
            TraceFlags::SAMPLED
        );

        // validate attributes.
        #[cfg(not(feature = "experimental_metadata_attributes"))]
        assert_eq!(log.record.attributes_iter().count(), 3);
        #[cfg(feature = "experimental_metadata_attributes")]
        assert_eq!(log.record.attributes_iter().count(), 7);
        assert!(attributes_contains(
            &log.record,
            &Key::new("event_id"),
            &AnyValue::Int(20.into())
        ));
        assert!(attributes_contains(
            &log.record,
            &Key::new("user_name"),
            &AnyValue::String("otel".into())
        ));
        assert!(attributes_contains(
            &log.record,
            &Key::new("user_email"),
            &AnyValue::String("otel@opentelemetry.io".into())
        ));
        #[cfg(feature = "experimental_metadata_attributes")]
        {
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.filename"),
                &AnyValue::String("layer.rs".into())
            ));
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.namespace"),
                &AnyValue::String("opentelemetry_appender_tracing::layer::tests".into())
            ));
            // The other 3 experimental_metadata_attributes are too unstable to check their value.
            // Ex.: The path will be different on a Windows and Linux machine.
            // Ex.: The line can change easily if someone makes changes in this source file.
            let attributes_key: Vec<Key> = log
                .record
                .attributes_iter()
                .map(|(key, _)| key.clone())
                .collect();
            assert!(attributes_key.contains(&Key::new("code.filepath")));
            assert!(attributes_key.contains(&Key::new("code.lineno")));
            assert!(!attributes_key.contains(&Key::new("log.target")));
        }
    }

    /*#[cfg(feature = "experimental_use_tracing_span_context")]
    #[test]
    fn tracing_appender_inside_tracing_crate_context() {
        use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;

        // Arrange
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        // setup tracing layer to compare trace/span IDs against
        let span_exporter = InMemorySpanExporterBuilder::new().build();
        let tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(span_exporter.clone())
            .build();
        let tracer = tracer_provider.tracer("test-tracer");

        let level_filter = tracing_subscriber::filter::LevelFilter::INFO;
        let log_layer =
            layer::OpenTelemetryTracingBridge::new(&logger_provider).with_filter(level_filter);

        let subscriber = tracing_subscriber::registry()
            .with(log_layer)
            .with(tracing_opentelemetry::layer().with_tracer(tracer));

        // Avoiding global subscriber.init() as that does not play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);

        // Act
        tracing::info_span!("outer-span").in_scope(|| {
            error!("first-event");

            tracing::info_span!("inner-span").in_scope(|| {
                error!("second-event");
            });
        });

        assert!(logger_provider.force_flush().is_ok());

        let logs = exporter.get_emitted_logs().expect("No emitted logs");
        assert_eq!(logs.len(), 2);

        let spans = span_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 2);

        let trace_id = spans[0].span_context.trace_id();
        assert_eq!(trace_id, spans[1].span_context.trace_id());
        let inner_span_id = spans[0].span_context.span_id();
        let outer_span_id = spans[1].span_context.span_id();
        assert_eq!(outer_span_id, spans[0].parent_span_id);

        let trace_ctx0 = logs[0].record.trace_context().unwrap();
        let trace_ctx1 = logs[1].record.trace_context().unwrap();

        assert_eq!(trace_ctx0.trace_id, trace_id);
        assert_eq!(trace_ctx1.trace_id, trace_id);
        assert_eq!(trace_ctx0.span_id, outer_span_id);
        assert_eq!(trace_ctx1.span_id, inner_span_id);
    }*/

    #[test]
    fn tracing_appender_standalone_with_tracing_log() {
        // Arrange
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let subscriber = create_tracing_subscriber(exporter.clone(), &logger_provider);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);
        drop(tracing_log::LogTracer::init());

        // Act
        log::error!(target: "my-system", "log from log crate");
        assert!(logger_provider.force_flush().is_ok());

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .first()
            .expect("Atleast one log is expected to be present.");

        // Validate common fields
        assert_eq!(log.instrumentation.name(), "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number(), Some(Severity::Error));

        // Validate trace context is none.
        assert!(log.record.trace_context().is_none());

        // Attributes can be polluted when we don't use this feature.
        #[cfg(feature = "experimental_metadata_attributes")]
        assert_eq!(log.record.attributes_iter().count(), 4);

        #[cfg(feature = "experimental_metadata_attributes")]
        {
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.filename"),
                &AnyValue::String("layer.rs".into())
            ));
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.namespace"),
                &AnyValue::String("opentelemetry_appender_tracing::layer::tests".into())
            ));
            // The other 3 experimental_metadata_attributes are too unstable to check their value.
            // Ex.: The path will be different on a Windows and Linux machine.
            // Ex.: The line can change easily if someone makes changes in this source file.
            let attributes_key: Vec<Key> = log
                .record
                .attributes_iter()
                .map(|(key, _)| key.clone())
                .collect();
            assert!(attributes_key.contains(&Key::new("code.filepath")));
            assert!(attributes_key.contains(&Key::new("code.lineno")));
            assert!(!attributes_key.contains(&Key::new("log.target")));
        }
    }

    #[test]
    fn tracing_appender_inside_tracing_context_with_tracing_log() {
        // Arrange
        let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();

        let subscriber = create_tracing_subscriber(exporter.clone(), &logger_provider);

        // avoiding setting tracing subscriber as global as that does not
        // play well with unit tests.
        let _guard = tracing::subscriber::set_default(subscriber);
        drop(tracing_log::LogTracer::init());

        // setup tracing as well.
        let tracer_provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .build();
        let tracer = tracer_provider.tracer("test-tracer");

        // Act
        let (trace_id_expected, span_id_expected) = tracer.in_span("test-span", |cx| {
            let trace_id = cx.span().span_context().trace_id();
            let span_id = cx.span().span_context().span_id();

            // logging is done inside span context.
            log::error!(target: "my-system", "log from log crate");
            (trace_id, span_id)
        });

        assert!(logger_provider.force_flush().is_ok());

        // Assert TODO: move to helper methods
        let exported_logs = exporter
            .get_emitted_logs()
            .expect("Logs are expected to be exported.");
        assert_eq!(exported_logs.len(), 1);
        let log = exported_logs
            .first()
            .expect("Atleast one log is expected to be present.");

        // validate common fields.
        assert_eq!(log.instrumentation.name(), "opentelemetry-appender-tracing");
        assert_eq!(log.record.severity_number(), Some(Severity::Error));

        // validate trace context.
        assert!(log.record.trace_context().is_some());
        assert_eq!(
            log.record.trace_context().unwrap().trace_id,
            trace_id_expected
        );
        assert_eq!(
            log.record.trace_context().unwrap().span_id,
            span_id_expected
        );
        assert_eq!(
            log.record.trace_context().unwrap().trace_flags.unwrap(),
            TraceFlags::SAMPLED
        );

        // Attributes can be polluted when we don't use this feature.
        #[cfg(feature = "experimental_metadata_attributes")]
        assert_eq!(log.record.attributes_iter().count(), 4);

        #[cfg(feature = "experimental_metadata_attributes")]
        {
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.filename"),
                &AnyValue::String("layer.rs".into())
            ));
            assert!(attributes_contains(
                &log.record,
                &Key::new("code.namespace"),
                &AnyValue::String("opentelemetry_appender_tracing::layer::tests".into())
            ));
            // The other 3 experimental_metadata_attributes are too unstable to check their value.
            // Ex.: The path will be different on a Windows and Linux machine.
            // Ex.: The line can change easily if someone makes changes in this source file.
            let attributes_key: Vec<Key> = log
                .record
                .attributes_iter()
                .map(|(key, _)| key.clone())
                .collect();
            assert!(attributes_key.contains(&Key::new("code.filepath")));
            assert!(attributes_key.contains(&Key::new("code.lineno")));
            assert!(!attributes_key.contains(&Key::new("log.target")));
        }
    }
}
