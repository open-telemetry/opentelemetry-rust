use async_trait::async_trait;
use eventheader::{FieldFormat, Level, Opcode};
use eventheader_dynamic::EventBuilder;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use opentelemetry::{logs::AnyValue, logs::Severity, Key};
use std::{cell::RefCell, str, time::SystemTime};

/// Provider group associated with the user_events exporter
pub type ProviderGroup = Option<Cow<'static, str>>;

thread_local! { static EBW: RefCell<EventBuilder> = RefCell::new(EventBuilder::new());}

/// Exporter config
#[derive(Debug)]
pub struct ExporterConfig {
    /// keyword associated with user_events name
    /// These should be mapped to logger_name as of now.
    pub keywords_map: HashMap<String, u64>,
    /// default keyword if map is not defined.
    pub default_keyword: u64,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            keywords_map: HashMap::new(),
            default_keyword: 1,
        }
    }
}

impl ExporterConfig {
    pub(crate) fn get_log_keyword(&self, name: &str) -> Option<u64> {
        self.keywords_map.get(name).copied()
    }

    pub(crate) fn get_log_keyword_or_default(&self, name: &str) -> Option<u64> {
        if self.keywords_map.is_empty() {
            Some(self.default_keyword)
        } else {
            self.get_log_keyword(name)
        }
    }
}
pub(crate) struct UserEventsExporter {
    provider: Arc<eventheader_dynamic::Provider>,
    exporter_config: ExporterConfig,
}

const EVENT_ID: &str = "event_id";
const EVENT_NAME_PRIMARY: &str = "event_name";
const EVENT_NAME_SECONDARY: &str = "name";

//TBD - How to configure provider name and provider group
impl UserEventsExporter {
    pub(crate) fn new(
        provider_name: &str,
        _provider_group: ProviderGroup,
        exporter_config: ExporterConfig,
    ) -> Self {
        let mut options = eventheader_dynamic::Provider::new_options();
        options = *options.group_name(provider_name);
        let mut eventheader_provider: eventheader_dynamic::Provider =
            eventheader_dynamic::Provider::new(provider_name, &options);
        Self::register_keywords(&mut eventheader_provider, &exporter_config);
        UserEventsExporter {
            provider: Arc::new(eventheader_provider),
            exporter_config,
        }
    }

    fn register_events(eventheader_provider: &mut eventheader_dynamic::Provider, keyword: u64) {
        let levels = [
            eventheader::Level::Informational,
            eventheader::Level::Verbose,
            eventheader::Level::Warning,
            eventheader::Level::Error,
            eventheader::Level::CriticalError,
        ];

        for &level in levels.iter() {
            eventheader_provider.register_set(level, keyword);
        }
    }

    fn register_keywords(
        eventheader_provider: &mut eventheader_dynamic::Provider,
        exporter_config: &ExporterConfig,
    ) {
        if exporter_config.keywords_map.is_empty() {
            println!(
                "Register default keyword {}",
                exporter_config.default_keyword
            );
            Self::register_events(eventheader_provider, exporter_config.default_keyword);
        }

        for keyword in exporter_config.keywords_map.values() {
            Self::register_events(eventheader_provider, *keyword);
        }
    }

    fn add_attribute_to_event(&self, eb: &mut EventBuilder, attrib: &(Key, AnyValue)) {
        let field_name = &attrib.0.to_string();
        match attrib.1.to_owned() {
            AnyValue::Boolean(b) => {
                eb.add_value(field_name, b, FieldFormat::Boolean, 0);
            }
            AnyValue::Int(i) => {
                eb.add_value(field_name, i, FieldFormat::SignedInt, 0);
            }
            AnyValue::Double(f) => {
                eb.add_value(field_name, f, FieldFormat::Float, 0);
            }
            AnyValue::String(s) => {
                eb.add_str(field_name, &s.to_string(), FieldFormat::Default, 0);
            }
            _ => (),
        }
    }

    fn get_severity_level(&self, severity: Severity) -> Level {
        match severity {
            Severity::Debug
            | Severity::Debug2
            | Severity::Debug3
            | Severity::Debug4
            | Severity::Trace
            | Severity::Trace2
            | Severity::Trace3
            | Severity::Trace4 => eventheader::Level::Verbose,

            Severity::Info | Severity::Info2 | Severity::Info3 | Severity::Info4 => {
                eventheader::Level::Informational
            }

            Severity::Error | Severity::Error2 | Severity::Error3 | Severity::Error4 => {
                eventheader::Level::Error
            }

            Severity::Fatal | Severity::Fatal2 | Severity::Fatal3 | Severity::Fatal4 => {
                eventheader::Level::CriticalError
            }

            Severity::Warn | Severity::Warn2 | Severity::Warn3 | Severity::Warn4 => {
                eventheader::Level::Warning
            }
        }
    }

    #[allow(dead_code)]
    fn enabled(&self, level: u8, keyword: u64) -> bool {
        let es = self.provider.find_set(level.into(), keyword);
        match es {
            Some(x) => x.enabled(),
            _ => false,
        };
        false
    }

    pub(crate) fn export_log_data(
        &self,
        log_data: &opentelemetry_sdk::export::logs::LogData,
    ) -> opentelemetry_sdk::export::logs::ExportResult {
        let mut level: Level = Level::Invalid;
        if log_data.record.severity_number.is_some() {
            level = self.get_severity_level(log_data.record.severity_number.unwrap());
        }

        let keyword = self
            .exporter_config
            .get_log_keyword_or_default(log_data.instrumentation.name.as_ref());

        if keyword.is_none() {
            return Ok(());
        }

        let log_es = if let Some(es) = self
            .provider
            .find_set(level.as_int().into(), keyword.unwrap())
        {
            es
        } else {
            return Ok(());
        };
        if log_es.enabled() {
            EBW.with(|eb| {
                let mut eb = eb.borrow_mut();
                let event_tags: u32 = 0; // TBD name and event_tag values
                eb.reset(log_data.instrumentation.name.as_ref(), event_tags as u16);
                eb.opcode(Opcode::Info);

                eb.add_value("__csver__", 0x0401u16, FieldFormat::HexInt, 0);

                // populate CS PartA
                let mut cs_a_count = 0;
                let event_time: SystemTime = log_data
                    .record
                    .timestamp
                    .unwrap_or(log_data.record.observed_timestamp);
                cs_a_count += 1; // for event_time
                eb.add_struct("PartA", cs_a_count, 0);
                {
                    let time: String = chrono::DateTime::to_rfc3339(
                        &chrono::DateTime::<chrono::Utc>::from(event_time),
                    );
                    eb.add_str("time", time, FieldFormat::Default, 0);
                }
                //populate CS PartC
                let (mut is_event_id, mut event_id) = (false, 0);
                let (mut is_event_name, mut event_name) = (false, "");

                if let Some(attr_list) = &log_data.record.attributes {
                    let (mut is_part_c_present, mut cs_c_bookmark, mut cs_c_count) = (false, 0, 0);
                    for attrib in attr_list.iter() {
                        match (attrib.0.as_str(), &attrib.1) {
                            (EVENT_ID, AnyValue::Int(value)) => {
                                is_event_id = true;
                                event_id = *value;
                                continue;
                            }
                            (EVENT_NAME_PRIMARY, AnyValue::String(value)) => {
                                is_event_name = true;
                                event_name = value.as_str();
                                continue;
                            }
                            (EVENT_NAME_SECONDARY, AnyValue::String(value)) => {
                                println!("Event name sec is {}", value.as_str());
                                if !is_event_name {
                                    event_name = value.as_str();
                                }
                                continue;
                            }
                            _ => {
                                if !is_part_c_present {
                                    eb.add_struct_with_bookmark("PartC", 1, 0, &mut cs_c_bookmark);
                                    is_part_c_present = true;
                                }
                                self.add_attribute_to_event(&mut eb, attrib);
                                cs_c_count += 1;
                            }
                        }
                    }

                    if is_part_c_present {
                        eb.set_struct_field_count(cs_c_bookmark, cs_c_count);
                    }
                }
                // populate CS PartB
                let mut cs_b_bookmark: usize = 0;
                let mut cs_b_count = 0;
                eb.add_struct_with_bookmark("PartB", 1, 0, &mut cs_b_bookmark);
                eb.add_str("_typeName", "Logs", FieldFormat::Default, 0);
                cs_b_count += 1;

                if log_data.record.body.is_some() {
                    eb.add_str(
                        "body",
                        match log_data.record.body.as_ref().unwrap() {
                            AnyValue::Int(value) => value.to_string(),
                            AnyValue::String(value) => value.to_string(),
                            AnyValue::Boolean(value) => value.to_string(),
                            AnyValue::Double(value) => value.to_string(),
                            AnyValue::Bytes(value) => String::from_utf8_lossy(value).to_string(),
                            AnyValue::ListAny(_value) => "".to_string(),
                            AnyValue::Map(_value) => "".to_string(),
                        },
                        FieldFormat::Default,
                        0,
                    );
                    cs_b_count += 1;
                }
                if level != Level::Invalid {
                    eb.add_value("severityNumber", level.as_int(), FieldFormat::SignedInt, 0);
                    cs_b_count += 1;
                }
                if log_data.record.severity_text.is_some() {
                    eb.add_str(
                        "severityText",
                        log_data.record.severity_text.as_ref().unwrap().as_ref(),
                        FieldFormat::SignedInt,
                        0,
                    );
                    cs_b_count += 1;
                }
                if is_event_id {
                    eb.add_value("eventId", event_id, FieldFormat::SignedInt, 0);
                    cs_b_count += 1;
                }
                if event_name.len() > 0 {
                    eb.add_str("name", event_name, FieldFormat::Default, 0);
                    cs_b_count += 1;
                }
                eb.set_struct_field_count(cs_b_bookmark, cs_b_count);

                eb.write(&log_es, None, None);
            });
            return Ok(());
        }
        Ok(())
    }
}

impl Debug for UserEventsExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("user_events log exporter")
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for UserEventsExporter {
    async fn export(
        &mut self,
        batch: Vec<opentelemetry_sdk::export::logs::LogData>,
    ) -> opentelemetry::logs::LogResult<()> {
        for log_data in batch {
            let _ = self.export_log_data(&log_data);
        }
        Ok(())
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, level: Severity, _target: &str, name: &str) -> bool {
        let (found, keyword) = if self.exporter_config.keywords_map.is_empty() {
            (true, self.exporter_config.default_keyword)
        } else {
            // TBD - target is not used as of now for comparison.
            match self.exporter_config.get_log_keyword(name) {
                Some(x) => (true, x),
                _ => (false, 0),
            }
        };
        if !found {
            return false;
        }
        let es = self
            .provider
            .find_set(self.get_severity_level(level), keyword);
        match es {
            Some(x) => x.enabled(),
            _ => false,
        }
    }
}
