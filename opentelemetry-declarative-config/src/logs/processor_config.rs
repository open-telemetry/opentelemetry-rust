//! # Log Processor Configuration module
//!
//! This module defines the configuration structures for log processors

use serde::Deserialize;
use serde_yaml::Value;

/// Configuration for Log Processors
#[derive(Deserialize)]
pub struct ProcessorConfig {
    pub batch: Option<ProcessorBatchConfig>,
}

/// Configuration for Batch Log Processor
#[derive(Deserialize)]
pub struct ProcessorBatchConfig {
    pub exporter: Value,
}
