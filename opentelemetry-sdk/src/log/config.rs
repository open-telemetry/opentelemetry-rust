use std::sync::Arc;

/// Log emitter configuration.
#[derive(Debug, Default)]
pub struct Config {
    /// Contains attributes representing an entity that produces telemetry.
    pub resource: Option<Arc<crate::Resource>>,
}
