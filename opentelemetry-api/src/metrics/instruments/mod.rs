use crate::metrics::{Meter, MetricsError, Result, Unit};
use core::fmt;
use std::convert::TryFrom;
use std::marker;

pub(super) mod counter;
pub(super) mod gauge;
pub(super) mod histogram;
pub(super) mod up_down_counter;

// instrument validation error strings
const INSTRUMENT_NAME_EMPTY: &str = "instrument name must be non-empty";
const INSTRUMENT_NAME_LENGTH: &str = "instrument name must be less than 64 characters";
const INSTRUMENT_NAME_INVALID_CHAR: &str =
    "characters in instrument name must be ASCII and belong to the alphanumeric characters, '_', '.', and '-'";
const INSTRUMENT_NAME_FIRST_ALPHABETIC: &str =
    "instrument name must start with an alphabetic character";
const INSTRUMENT_UNIT_LENGTH: &str = "instrument unit must be less than 64 characters";
const INSTRUMENT_UNIT_INVALID_CHAR: &str = "characters in instrument unit must be ASCII";

/// Configuration for building an instrument.
pub struct InstrumentBuilder<'a, T> {
    meter: &'a Meter,
    name: String,
    description: Option<String>,
    unit: Option<Unit>,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> InstrumentBuilder<'a, T>
where
    T: TryFrom<Self, Error = MetricsError>,
{
    /// Create a new instrument builder
    pub(crate) fn new(meter: &'a Meter, name: String) -> Self {
        InstrumentBuilder {
            meter,
            name,
            description: None,
            unit: None,
            _marker: marker::PhantomData,
        }
    }

    /// Set the description for this instrument
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the unit for this instrument.
    ///
    /// Unit is case sensitive(`kb` is not the same as `kB`).
    ///
    /// Unit must be:
    /// - ASCII string
    /// - No longer than 63 characters
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Validate the instrument configuration and creates a new instrument.
    pub fn try_init(self) -> Result<T> {
        self.validate_instrument_config()
            .map_err(MetricsError::InvalidInstrumentConfiguration)?;
        T::try_from(self)
    }

    /// Creates a new instrument.
    ///
    /// # Panics
    ///
    /// This function panics if the instrument configuration is invalid or the instrument cannot be created. Use [`try_init`] if you want to
    /// handle errors.
    pub fn init(self) -> T {
        self.try_init().unwrap()
    }

    fn validate_instrument_config(&self) -> std::result::Result<(), &'static str> {
        // validate instrument name
        if self.name.is_empty() {
            return Err(INSTRUMENT_NAME_EMPTY);
        }
        if self.name.len() > 63 {
            return Err(INSTRUMENT_NAME_LENGTH);
        }
        if self.name.starts_with(|c: char| !c.is_ascii_alphabetic()) {
            return Err(INSTRUMENT_NAME_FIRST_ALPHABETIC);
        }
        if self
            .name
            .contains(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.' && c != '-')
        {
            return Err(INSTRUMENT_NAME_INVALID_CHAR);
        }

        // validate instrument unit
        if let Some(unit) = &self.unit {
            if unit.as_str().len() > 63 {
                return Err(INSTRUMENT_UNIT_LENGTH);
            }
            if unit.as_str().contains(|c: char| !c.is_ascii()) {
                return Err(INSTRUMENT_UNIT_INVALID_CHAR);
            }
        }
        Ok(())
    }
}

impl<T> fmt::Debug for InstrumentBuilder<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentBuilder")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .field("kind", &std::any::type_name::<T>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::instruments::{
        INSTRUMENT_NAME_FIRST_ALPHABETIC, INSTRUMENT_NAME_INVALID_CHAR, INSTRUMENT_NAME_LENGTH,
        INSTRUMENT_UNIT_INVALID_CHAR, INSTRUMENT_UNIT_LENGTH,
    };
    use crate::metrics::noop::NoopMeterCore;
    use crate::metrics::{Counter, InstrumentBuilder, Unit};
    use crate::InstrumentationLibrary;
    use std::sync::Arc;

    #[test]
    fn test_instrument_config_validation() {
        let meter = crate::metrics::Meter::new(
            InstrumentationLibrary::default(),
            Arc::new(NoopMeterCore::new()),
        );
        // (name, expected error)
        let instrument_name_test_cases = vec![
            ("validateName", ""),
            ("_startWithNoneAlphabet", INSTRUMENT_NAME_FIRST_ALPHABETIC),
            ("utf8char锈", INSTRUMENT_NAME_INVALID_CHAR),
            (
                "a12345678901234567890123456789012345678901234567890123456789012",
                "",
            ),
            (
                "a123456789012345678901234567890123456789012345678901234567890123",
                INSTRUMENT_NAME_LENGTH,
            ),
            ("invalid name", INSTRUMENT_NAME_INVALID_CHAR),
        ];
        for (name, expected_error) in instrument_name_test_cases {
            let builder: InstrumentBuilder<'_, Counter<u64>> =
                InstrumentBuilder::new(&meter, name.to_string());
            if expected_error.is_empty() {
                assert!(builder.validate_instrument_config().is_ok());
            } else {
                assert_eq!(
                    builder.validate_instrument_config().unwrap_err(),
                    expected_error
                );
            }
        }

        // (unit, expected error)
        let instrument_unit_test_cases = vec![
            (
                "0123456789012345678901234567890123456789012345678901234567890123",
                INSTRUMENT_UNIT_LENGTH,
            ),
            ("utf8char锈", INSTRUMENT_UNIT_INVALID_CHAR),
            ("kb", ""),
        ];

        for (unit, expected_error) in instrument_unit_test_cases {
            let builder: InstrumentBuilder<'_, Counter<u64>> =
                InstrumentBuilder::new(&meter, "test".to_string()).with_unit(Unit::new(unit));
            if expected_error.is_empty() {
                assert!(builder.validate_instrument_config().is_ok());
            } else {
                assert_eq!(
                    builder.validate_instrument_config().unwrap_err(),
                    expected_error
                );
            }
        }
    }
}
