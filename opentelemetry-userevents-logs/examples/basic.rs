#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

//! run with `$ cargo run --example basic --all-features

use opentelemetry_api::{
    logs::LoggerProvider as _,
    logs::LogRecordBuilder,
    logs::Logger,
    logs::Severity,
    trace::{Span, Tracer, TracerProvider as _, mark_span_as_active},
    Context, KeyValue, logs::AnyValue
};

use opentelemetry_sdk::{
    logs::LoggerProvider,
    runtime,
    trace::TracerProvider,
};
use log::{Log, info, error, log, debug, Level};

use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_userevents_logs::{ExporterConfig, ReentrantLogProcessor, ProviderGroup};
use std::time::SystemTime;
use std::thread;

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig{keyword : 1};
    let reenterant_processor = ReentrantLogProcessor::new("test", None , exporter_config);
    LoggerProvider::builder()
    .with_log_processor(reenterant_processor).build()
}

fn main() {

    // Example with log crate appender.
    let logger_provider = init_logger();
    let logger = OpenTelemetryLogBridge::new(&logger_provider);
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(log::LevelFilter::Info);
    info!("test");

    // Example with LogBridge API - this is NOT supposed to be used by end user
    let logger_provider = init_logger();
    let logger: opentelemetry_sdk::logs::Logger = logger_provider.logger("test");
    let num_of_messages = 1;
    let mut messages: Vec<String> = vec![String::new(); num_of_messages+1];
    for i in 0..num_of_messages {
        messages[i] = i.to_string().to_owned() + "_body";
    }
    let handle = thread::spawn( move || {
    for i in 0..num_of_messages {
        let log_record = opentelemetry_api::logs::LogRecordBuilder::new()
            .with_body(messages[i].to_owned().into())
            .with_severity_number(Severity::Debug)
            .with_attribute("key1", "value1")
            .with_attribute("event_id", 23)
            .with_attribute("event_name", "test_event")
            .with_timestamp(SystemTime::now())
            .build();
        logger.emit(log_record);
    }
    });
    handle.join().unwrap();

}
