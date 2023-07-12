use async_trait::async_trait;
use eventheader::{FieldFormat, Level, Opcode};
use eventheader_dynamic::EventBuilder;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use opentelemetry_api::{logs::AnyValue, logs::Severity, Key};
use std::{cell::RefCell, str, time::SystemTime};

/// Provider group associated with the user_events exporter
pub type ProviderGroup = Option<Cow<'static, str>>;

thread_local! { static EBW: RefCell<EventBuilder> = RefCell::new(EventBuilder::new());}

/// Exporter config
#[derive(Debug)]
pub struct ExporterConfig {
    /// keyword associated with user_events name
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
const EVENT_NAME: &str = "event_name";

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
        if exporter_config.keywords_map.is_empty() {
            println!(
                "Register default keyword {}",
                exporter_config.default_keyword
            );
            Self::register_events(&mut eventheader_provider, exporter_config.default_keyword)
        }
        for keyword in exporter_config.keywords_map.values() {
            Self::register_events(&mut eventheader_provider, *keyword)
        }
        UserEventsExporter {
            provider: Arc::new(eventheader_provider),
            exporter_config,
        }
    }

    fn register_events(eventheader_provider: &mut eventheader_dynamic::Provider, keyword: u64) {
        eventheader_provider.register_set(eventheader::Level::Informational, keyword);
        eventheader_provider.register_set(eventheader::Level::Verbose, keyword);
        eventheader_provider.register_set(eventheader::Level::Warning, keyword);
        eventheader_provider.register_set(eventheader::Level::Error, keyword);
        eventheader_provider.register_set(eventheader::Level::CriticalError, keyword);
    }

    fn add_attributes_to_event(
        &self,
        eb: &mut EventBuilder,
        attribs: &mut dyn Iterator<Item = (&Key, &AnyValue)>,
    ) {
        for attrib in attribs {
            if attrib.0.to_string() == EVENT_ID || attrib.0.to_string() == EVENT_NAME {
                continue;
            }
            let field_name = &attrib.0.to_string();
            match attrib.1 {
                AnyValue::Boolean(b) => {
                    eb.add_value(field_name, *b, FieldFormat::Boolean, 0);
                }
                AnyValue::Int(i) => {
                    eb.add_value(field_name, *i, FieldFormat::SignedInt, 0);
                }
                AnyValue::Double(f) => {
                    eb.add_value(field_name, *f, FieldFormat::Float, 0);
                }
                AnyValue::String(s) => {
                    eb.add_str(field_name, &s.to_string(), FieldFormat::Default, 0);
                }
                _ => (),
            }
        }
    }

    fn get_serverity_level(&self, severity: Severity) -> Level {
        let level: Level = match severity {
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
        };
        level
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
            level = self.get_serverity_level(log_data.record.severity_number.unwrap());
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
                let [mut cs_a_count, mut cs_b_count, mut cs_c_count] = [0; 3];
                let mut eb = eb.borrow_mut();
                let event_tags: u32 = 0; // TBD name and event_tag values
                eb.reset(log_data.instrumentation.name.as_ref(), event_tags as u16);
                eb.opcode(Opcode::Info);

                eb.add_value("__csver__", 0x0401u16, FieldFormat::HexInt, 0);

                // populate CS PartA
                let event_time: SystemTime;
                if log_data.record.timestamp.is_some() {
                    event_time = log_data.record.timestamp.unwrap();
                } else if log_data.record.observed_timestamp.is_some() {
                    event_time = log_data.record.observed_timestamp.unwrap();
                } else {
                    event_time = SystemTime::now();
                }
                cs_a_count += 1; // for event_time
                eb.add_struct("PartA", cs_a_count, 0);
                {
                    let time: String = chrono::DateTime::to_rfc3339(
                        &chrono::DateTime::<chrono::Utc>::from(event_time),
                    );
                    eb.add_str("time", time, FieldFormat::Default, 0);
                }

                // populate CS PartB
                // Get Event_Id and Event_Name if present.
                let (mut event_id, mut event_name) = (0, "");
                let mut event_count = 0;
                if log_data.record.attributes.is_some() {
                    for (k, v) in log_data.record.attributes.as_ref().unwrap().into_iter() {
                        if k.as_str() == EVENT_ID {
                            event_id = match v {
                                AnyValue::Int(value) => {
                                    event_count += 1;
                                    *value
                                }
                                _ => 0,
                            }
                        }
                        if k.as_str() == EVENT_NAME {
                            event_name = match v {
                                AnyValue::String(value) => {
                                    event_count += 1;
                                    value.as_ref()
                                }
                                _ => "",
                            }
                        }
                    }
                }
                cs_b_count += event_count;
                // check body, severity number and severity text
                let (mut is_body_present, mut is_severity_text_present) = (false, false);
                if log_data.record.body.is_some() {
                    cs_b_count += 1;
                    is_body_present = true;
                }
                if level != Level::Invalid {
                    cs_b_count += cs_b_count;
                }
                if log_data.record.severity_text.is_some() {
                    cs_b_count += cs_b_count;
                    is_severity_text_present = true;
                }

                if cs_b_count > 0 {
                    eb.add_struct("PartB", cs_b_count, 0);
                    {
                        if level != Level::Invalid {
                            eb.add_value(
                                "severityNumber",
                                level.as_int(),
                                FieldFormat::SignedInt,
                                0,
                            );
                        }
                        if is_severity_text_present {
                            eb.add_str(
                                "severityText",
                                log_data.record.severity_text.as_ref().unwrap().as_ref(),
                                FieldFormat::SignedInt,
                                0,
                            );
                        }
                        if is_body_present {
                            eb.add_str(
                                "body",
                                match log_data.record.body.as_ref().unwrap() {
                                    AnyValue::Int(value) => value.to_string(),
                                    AnyValue::String(value) => value.to_string(),
                                    AnyValue::Boolean(value) => value.to_string(),
                                    AnyValue::Double(value) => value.to_string(),
                                    AnyValue::Bytes(value) => {
                                        String::from_utf8_lossy(value).to_string()
                                    }
                                    AnyValue::ListAny(_value) => "".to_string(),
                                    AnyValue::Map(_value) => "".to_string(),
                                },
                                FieldFormat::Default,
                                0,
                            );
                        }
                        if event_id > 0 {
                            eb.add_value("eventId", event_id, FieldFormat::SignedInt, 0);
                        }
                        if !event_name.is_empty() {
                            eb.add_str("name", event_name, FieldFormat::Default, 0);
                        }
                    };
                }

                // populate CS PartC
                if log_data.record.attributes.is_some() {
                    cs_c_count =
                        log_data.record.attributes.as_ref().unwrap().len() as u8 - event_count;
                }
                if cs_c_count > 0 {
                    eb.add_struct("PartC", cs_c_count, 0);
                    {
                        self.add_attributes_to_event(
                            &mut eb,
                            &mut log_data.record.attributes.as_ref().unwrap().iter(),
                        );
                    }
                }
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
    ) -> opentelemetry_api::logs::LogResult<()> {
        for log_data in batch {
            let _ = self.export_log_data(&log_data);
        }
        Ok(())
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, name: &str, level: Severity) -> bool {
        //print!("LALIT:event-enabled check for {} and {:?}", name, level);

        let (found, keyword) = if self.exporter_config.keywords_map.len() == 0 {
            (true, self.exporter_config.default_keyword)
        } else {
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
            .find_set(self.get_serverity_level(level), keyword);
        match es {
            Some(x) => x.enabled(),
            _ => false,
        };
        false
    }
}
