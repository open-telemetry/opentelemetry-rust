#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

//! run with `$ cargo run --example basic --all-features

use opentelemetry_api::{
    logs::LoggerProvider as _,
    logs::LogRecordBuilder,
    logs::Logger,
    logs::Severity,
    trace::{Span, Tracer, TracerProvider as _, mark_span_as_active},
    Context, KeyValue,
};

use opentelemetry_sdk::{
    logs::LoggerProvider,
    runtime,
    trace::TracerProvider,
};

use opentelemetry_userevents_exporter_log::{ExporterConfig, RealTimeLogProcessor, ProviderGroup};
use std::time::SystemTime;

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig{keyword : 1};
    let prov_group = Some("prov_group");

    let realtime_processor = RealTimeLogProcessor::new("test12345", None , exporter_config);

    LoggerProvider::builder()
    .with_log_processor(realtime_processor).build()
}

fn main() {
    let logger_provider = init_logger();
    let prov_group = Some("test1234");
    let logger: opentelemetry_sdk::logs::Logger = logger_provider.logger("test");

    let log_record = opentelemetry_api::logs::LogRecordBuilder::new().with_body("test".into()).with_severity_number(Severity::Debug)
        .with_attribute("key1", "value1")
        .with_attribute("event_id", 23)
        .with_attribute("event_name", "test_event")
        .with_timestamp(SystemTime::now())
        .build();
    logger.emit(log_record);
    println!("Emission done...\n");
}
