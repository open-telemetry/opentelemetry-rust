//! # Logs Exporters module.
//!
//! This module contains the definitions and common implementations for various logs exporters
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected logs data to different backends or systems.

pub mod otlp_batch_exporter;
pub mod stdout_batch_exporter;
