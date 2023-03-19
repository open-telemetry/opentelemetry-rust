pub use opentelemetry_api::InstrumentationLibrary;

/// A logical unit of the application code with which the emitted telemetry can
/// be associated.
pub type Scope = InstrumentationLibrary;
