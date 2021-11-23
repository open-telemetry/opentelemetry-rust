use crate::metrics::Unit;
use crate::sdk::InstrumentationLibrary;
use std::borrow::Cow;

/// Config contains some options for metrics of any kind.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InstrumentConfig {
    pub(crate) description: Option<String>,
    pub(crate) unit: Option<Unit>,
    pub(crate) instrumentation_library: InstrumentationLibrary,
}

impl InstrumentConfig {
    /// Create a new config from instrumentation name
    pub fn with_instrumentation_name(instrumentation_name: &'static str) -> Self {
        InstrumentConfig {
            description: None,
            unit: None,
            instrumentation_library: InstrumentationLibrary::new(instrumentation_name, None),
        }
    }

    /// Create a new config with instrumentation name and optional version
    pub fn with_instrumentation<T: Into<Cow<'static, str>>>(
        instrumentation_name: T,
        instrumentation_version: Option<T>,
    ) -> Self {
        InstrumentConfig {
            description: None,
            unit: None,
            instrumentation_library: InstrumentationLibrary::new(
                instrumentation_name,
                instrumentation_version,
            ),
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
    pub fn instrumentation_name(&self) -> Cow<'static, str> {
        self.instrumentation_library.name.clone()
    }

    /// Instrumentation version returns the version of instrumentation
    pub fn instrumentation_version(&self) -> Option<Cow<'static, str>> {
        self.instrumentation_library.version.clone()
    }
}
