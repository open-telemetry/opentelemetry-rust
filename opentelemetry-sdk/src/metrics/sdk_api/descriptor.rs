use crate::metrics::sdk_api::{InstrumentKind, NumberKind};
use fnv::FnvHasher;
use opentelemetry_api::metrics::Unit;
use std::hash::{Hash, Hasher};

/// Descriptor contains all the settings that describe an instrument, including
/// its name, metric kind, number kind, and the configurable options.
#[derive(Clone, Debug, PartialEq)]
pub struct Descriptor {
    name: String,
    instrument_kind: InstrumentKind,
    number_kind: NumberKind,
    description: Option<String>,
    unit: Option<Unit>,
    attribute_hash: u64,
}

impl Descriptor {
    /// Create a new descriptor
    pub fn new(
        name: String,
        instrument_kind: InstrumentKind,
        number_kind: NumberKind,
        description: Option<String>,
        unit: Option<Unit>,
    ) -> Self {
        let mut hasher = FnvHasher::default();
        name.hash(&mut hasher);
        instrument_kind.hash(&mut hasher);
        number_kind.hash(&mut hasher);
        if let Some(description) = &description {
            description.hash(&mut hasher);
        }
        if let Some(unit) = &unit {
            unit.hash(&mut hasher);
        }

        Descriptor {
            name,
            instrument_kind,
            number_kind,
            description,
            unit,
            attribute_hash: hasher.finish(),
        }
    }

    /// The metric instrument's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// The specific kind of instrument.
    pub fn instrument_kind(&self) -> &InstrumentKind {
        &self.instrument_kind
    }

    /// NumberKind returns whether this instrument is declared over int64, float64, or uint64
    /// values.
    pub fn number_kind(&self) -> &NumberKind {
        &self.number_kind
    }

    /// A human-readable description of the metric instrument.
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Assign a new description
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// Unit describes the units of the metric instrument.
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|unit| unit.as_ref())
    }

    /// The pre-computed hash of the descriptor data
    pub fn attribute_hash(&self) -> u64 {
        self.attribute_hash
    }
}
