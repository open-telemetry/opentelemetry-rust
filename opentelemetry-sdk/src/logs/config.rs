use std::borrow::Cow;

/// Log emitter configuration.
#[derive(Debug, Default)]
pub struct Config {
    /// Contains attributes representing an entity that produces telemetry.
    pub resource: Cow<'static, crate::Resource>,
}
