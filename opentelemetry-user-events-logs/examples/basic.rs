//! run with `$ cargo run --example basic --all-features

use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_user_events_logs::{ExporterConfig, ReentrantLogProcessor};
use std::collections::HashMap;
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use ctrlc;
use std::sync::{Arc, Mutex};

use std::thread::sleep;
use std::time::Duration;

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig {
        default_keyword: 1,
        keywords_map: HashMap::new(),
    };
    let reenterant_processor = ReentrantLogProcessor::new("test", None, exporter_config);
    LoggerProvider::builder()
        .with_log_processor(reenterant_processor)
        .build()
}

fn main() {

    // Example with tracing appender.
    let logger_provider = init_logger();
    let user_events_layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);

    tracing_subscriber::registry().with(user_events_layer).init();

    // event_name is now passed as an attribute, but once https://github.com/tokio-rs/tracing/issues/1426
    // is done, it can be passed with name:"my-event-name", so it'll be available as metadata for
    // fast filtering.
    // event_id is also passed as an attribute now, there is nothing in metadata where a
    // numeric id can be stored.
    let mut event_id = 0;
    let error_no = Arc::new(Mutex::new(0));
    let info_no = Arc::new(Mutex::new(0));    
    println!("Generating logs...Press Ctrl-C to stop.");
    let er = error_no.clone();
    let i = info_no.clone();
    ctrlc::set_handler(move || {
        let er = er.lock().unwrap();
        let i = i.lock().unwrap();
        println!(
            "Generated {} error logs and {} info logs",
            er, i
        );
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
    loop {
        let event_name = format!("my-event-name {}", event_id);
        error!(
            event_name = event_name,
            event_id = event_id,
            user_name = "otel user",
            user_email = "otel@opentelemetry.io",
            "login failed"
        );
        *error_no.lock().unwrap() += 1;
        info!(
            event_name = event_name,
            event_id = event_id,
            user_name = "otel user",
            user_email = "otel@opentelemetry.io",
            "login successful"
        );
        *info_no.lock().unwrap() += 1;
        event_id += 1;
        sleep(Duration::from_secs(1));
    }
}
