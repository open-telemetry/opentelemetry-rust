//! # Metrics Reader Configuration module
//!
//! This module defines the configuration structures for metrics readers
//! used in OpenTelemetry SDKs. Readers are responsible for collecting
//! metrics data and exporting it to various backends or systems.

use serde::Deserialize;
use serde_yaml::Value;

/// Configuration for Metrics Readers
#[derive(Deserialize)]
pub struct ReaderConfig {
    pub pull: Option<PullReaderConfig>,
    pub periodic: Option<PeriodicReaderConfig>,
}

/// Configuration for Periodic Metrics Reader
#[derive(Deserialize)]
pub struct PeriodicReaderConfig {
    pub exporter: Value,
}

/// Configuration for Pull Metrics Reader
#[derive(Deserialize)]
pub struct PullReaderConfig {
    pub exporter: Value,
}
