//! # OpenTelemetry Log SDK

mod config;
mod log_emitter;
mod log_processor;

pub use config::Config;
pub use log_emitter::{Builder, Logger, LoggerProvider};
pub use log_processor::{
    BatchConfig, BatchLogProcessor, BatchLogProcessorBuilder, BatchMessage, LogProcessor,
    SimpleLogProcessor,
};
