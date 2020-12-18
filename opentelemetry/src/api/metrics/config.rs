use crate::Unit;

/// Config contains some options for metrics of any kind.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InstrumentConfig {
    pub(crate) description: Option<String>,
    pub(crate) unit: Option<Unit>,
    pub(crate) instrumentation_name: String,
    pub(crate) instrumentation_version: Option<String>,
}

impl InstrumentConfig {
    /// Create a new config from instrumentation name
    pub fn with_instrumentation_name(instrumentation_name: String) -> Self {
        InstrumentConfig {
            description: None,
            unit: None,
            instrumentation_name,
            instrumentation_version: None,
        }
    }

    /// Create a new config with instrumentation name and version
    pub fn with_instrumentation(
        instrumentation_name: String,
        instrumentation_version: String,
    ) -> Self {
        InstrumentConfig {
            description: None,
            unit: None,
            instrumentation_name,
            instrumentation_version: Some(instrumentation_version),
        }
    }

    /// Description is an optional field describing the metric instrument.
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Unit is an optional field describing the metric instrument data.
    pub fn unit(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    /// Instrumentation name is the name given to the Meter that created this instrument.
    pub fn instrumentation_name(&self) -> &String {
        &self.instrumentation_name
    }

    /// Instrumentation version returns the version of instrumentation
    pub fn instrumentation_version(&self) -> Option<&String> {
        self.instrumentation_version.as_ref()
    }
}
