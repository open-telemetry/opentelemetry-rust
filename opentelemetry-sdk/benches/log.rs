use std::collections::HashMap;
use std::time::SystemTime;

use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

use opentelemetry::logs::{
    AnyValue, LogRecord as _, LogResult, Logger as _, LoggerProvider as _, Severity,
};
use opentelemetry::trace::Tracer;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::Key;
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use opentelemetry_sdk::logs::{Logger, LoggerProvider};
use opentelemetry_sdk::trace::{config, Sampler, TracerProvider};

#[derive(Debug)]
struct VoidExporter;

#[async_trait]
impl LogExporter for VoidExporter {
    async fn export(&mut self, _batch: Vec<LogData>) -> LogResult<()> {
        LogResult::Ok(())
    }
}

fn log_benchmark_group<F: Fn(&Logger)>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("no-context", |b| {
        let provider = LoggerProvider::builder()
            .with_simple_exporter(VoidExporter)
            .build();

        let logger = provider.logger("no-context");

        b.iter(|| f(&logger));
    });

    group.bench_function("with-context", |b| {
        let provider = LoggerProvider::builder()
            .with_simple_exporter(VoidExporter)
            .build();

        let logger = provider.logger("with-context");

        // setup tracing as well.
        let tracer_provider = TracerProvider::builder()
            .with_config(config().with_sampler(Sampler::AlwaysOn))
            .build();
        let tracer = tracer_provider.tracer("bench-tracer");

        // Act
        tracer.in_span("bench-span", |_cx| {
            b.iter(|| f(&logger));
        });
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    log_benchmark_group(c, "simple-log", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        logger.emit(log_record);
    });

    log_benchmark_group(c, "simple-log-with-int", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testint", 2);
        logger.emit(log_record);
    });

    log_benchmark_group(c, "simple-log-with-double", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testdouble", 2.2);
        logger.emit(log_record);
    });

    log_benchmark_group(c, "simple-log-with-string", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("teststring", "test");
        logger.emit(log_record);
    });

    log_benchmark_group(c, "simple-log-with-bool", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testbool", AnyValue::Boolean(true));
        logger.emit(log_record);
    });

    let bytes = AnyValue::Bytes(vec![25u8, 30u8, 40u8]);
    log_benchmark_group(c, "simple-log-with-bytes", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testbytes", bytes.clone());
        logger.emit(log_record);
    });

    let bytes = AnyValue::Bytes(vec![
        25u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8,
        30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8,
        40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8,
        30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8,
        40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8,
        30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8, 30u8, 40u8,
    ]);
    log_benchmark_group(c, "simple-log-with-a-lot-of-bytes", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testbytes", bytes.clone());
        logger.emit(log_record);
    });

    let vec_any_values = AnyValue::ListAny(vec![AnyValue::Int(25), "test".into(), true.into()]);
    log_benchmark_group(c, "simple-log-with-vec-any-value", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testvec", vec_any_values.clone());
        logger.emit(log_record);
    });

    let vec_any_values = AnyValue::ListAny(vec![AnyValue::Int(25), "test".into(), true.into()]);
    let vec_any_values = AnyValue::ListAny(vec![
        AnyValue::Int(25),
        "test".into(),
        true.into(),
        vec_any_values,
    ]);
    log_benchmark_group(c, "simple-log-with-inner-vec-any-value", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testvec", vec_any_values.clone());
        logger.emit(log_record);
    });

    let map_any_values = AnyValue::Map(HashMap::from([
        ("testint".into(), 2.into()),
        ("testdouble".into(), 2.2.into()),
        ("teststring".into(), "test".into()),
    ]));
    log_benchmark_group(c, "simple-log-with-map-any-value", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testmap", map_any_values.clone());
        logger.emit(log_record);
    });

    let map_any_values = AnyValue::Map(HashMap::from([
        ("testint".into(), 2.into()),
        ("testdouble".into(), 2.2.into()),
        ("teststring".into(), "test".into()),
    ]));
    let map_any_values = AnyValue::Map(HashMap::from([
        ("testint".into(), 2.into()),
        ("testdouble".into(), 2.2.into()),
        ("teststring".into(), "test".into()),
        ("testmap".into(), map_any_values),
    ]));
    log_benchmark_group(c, "simple-log-with-inner-map-any-value", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("simple log".into());
        log_record.add_attribute("testmap", map_any_values.clone());
        logger.emit(log_record);
    });

    log_benchmark_group(c, "long-log", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Gravida in fermentum et sollicitudin ac orci phasellus. Ullamcorper dignissim cras tincidunt lobortis feugiat vivamus at augue. Magna etiam tempor orci eu. Sed tempus urna et pharetra pharetra massa.".into());
        logger.emit(log_record);
    });

    let now = SystemTime::now();
    log_benchmark_group(c, "full-log", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("full log".into());
        log_record.set_timestamp(now);
        log_record.set_observed_timestamp(now);
        log_record.set_severity_number(Severity::Warn);
        log_record.set_severity_text(Severity::Warn.name().into());
        logger.emit(log_record);
    });

    log_benchmark_group(c, "full-log-with-4-attributes", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("full log".into());
        log_record.set_timestamp(now);
        log_record.set_observed_timestamp(now);
        log_record.set_severity_number(Severity::Warn);
        log_record.set_severity_text(Severity::Warn.name().into());
        log_record.add_attribute("name", "my-event-name");
        log_record.add_attribute("event.id", 20);
        log_record.add_attribute("user.name", "otel");
        log_record.add_attribute("user.email", "otel@opentelemetry.io");
        logger.emit(log_record);
    });

    log_benchmark_group(c, "full-log-with-9-attributes", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("full log".into());
        log_record.set_timestamp(now);
        log_record.set_observed_timestamp(now);
        log_record.set_severity_number(Severity::Warn);
        log_record.set_severity_text(Severity::Warn.name().into());
        log_record.add_attribute("name", "my-event-name");
        log_record.add_attribute("event.id", 20);
        log_record.add_attribute("user.name", "otel");
        log_record.add_attribute("user.email", "otel@opentelemetry.io");
        log_record.add_attribute("code.filename", "log.rs");
        log_record.add_attribute("code.filepath", "opentelemetry_sdk/benches/log.rs");
        log_record.add_attribute("code.lineno", 96);
        log_record.add_attribute("code.namespace", "opentelemetry_sdk::benches::log");
        log_record.add_attribute("log.target", "opentelemetry_sdk::benches::log");
        logger.emit(log_record);
    });

    let attributes: Vec<(Key, AnyValue)> = vec![
        ("name".into(), "my-event-name".into()),
        ("event-id".into(), 20.into()),
        ("user.name".into(), "otel".into()),
        ("user.email".into(), "otel@opentelemetry.io".into()),
        ("code.filename".into(), "log.rs".into()),
        (
            "code.filepath".into(),
            "opentelemetry_sdk/benches/log.rs".into(),
        ),
        ("code.lineno".into(), 96.into()),
        (
            "code.namespace".into(),
            "opentelemetry_sdk::benches::log".into(),
        ),
        (
            "log.target".into(),
            "opentelemetry_sdk::benches::log".into(),
        ),
    ];
    log_benchmark_group(c, "full-log-with-attributes", |logger| {
        let mut log_record = logger.create_log_record();
        log_record.set_body("full log".into());
        log_record.set_timestamp(now);
        log_record.set_observed_timestamp(now);
        log_record.set_severity_number(Severity::Warn);
        log_record.set_severity_text(Severity::Warn.name().into());
        log_record.add_attributes(attributes.clone());
        logger.emit(log_record);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
