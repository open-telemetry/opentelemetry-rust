//! run with `$ cargo run --example basic --all-features

use opentelemetry_api::{
    logs::Logger,
    logs::LoggerProvider as _,
    logs::Severity,
};

use log::info;
use opentelemetry_sdk::logs::LoggerProvider;

use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_user_events_logs::{ExporterConfig, ReentrantLogProcessor};
use std::time::SystemTime;

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig { keyword: 1 };
    let reenterant_processor = ReentrantLogProcessor::new("test", None, exporter_config);
    LoggerProvider::builder()
        .with_log_processor(reenterant_processor)
        .build()
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
    let log_record = opentelemetry_api::logs::LogRecordBuilder::new()
        .with_body("test message".into())
        .with_severity_number(Severity::Debug)
        .with_attribute("key1", "value1")
        .with_attribute("event_id", 23)
        .with_attribute("event_name", "test_event")
        .with_timestamp(SystemTime::now())
        .build();
    logger.emit(log_record);
}
