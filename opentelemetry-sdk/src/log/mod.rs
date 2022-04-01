//! # OpenTelemetry Log SDK

mod config;
mod log_emitter;
mod log_processor;
mod record;
mod runtime;

pub use config::Config;
pub use log_emitter::{Builder, LogEmitter, LogEmitterProvider};
pub use log_processor::{
    BatchConfig, BatchLogProcessor, BatchLogProcessorBuilder, BatchMessage, LogProcessor,
    SimpleLogProcessor,
};
pub use record::{Any, LogRecord, LogRecordBuilder, Severity, TraceContext};
pub use runtime::{LogRuntime, TrySend};
