#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

use eventheader::{FieldFormat, Level, Opcode};
use eventheader_dynamic::{EventBuilder, EventSet};
//use crate::exporter_traits::*;
use async_trait::async_trait;

use std::any::Any;
use std::fs::File;
use std::sync::Arc;
use std::fmt::Debug;
use std::borrow::Cow;
use chrono::{Datelike, Timelike};


use std::{cell::RefCell, time::SystemTime, str};
use opentelemetry_api::{logs::Severity, logs::AnyValue, Array, Key, Value};

pub type ProviderGroup = Option<Cow<'static, str>>;

thread_local! { static EBW: RefCell<EventBuilder> = RefCell::new(EventBuilder::new());}

pub struct ExporterConfig{
    pub keyword: u64
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig { keyword: 1}
    }
}

impl ExporterConfig {
    pub(crate) fn get_log_event_keyword(&self) -> u64{
        self.keyword
    }
}


pub(crate) struct UserEventsExporter {
    provider: Arc<eventheader_dynamic::Provider>,
    exporter_config: ExporterConfig,
}

impl UserEventsExporter {
    pub(crate) fn new(
        provider_name: &str,
        provider_group: ProviderGroup,
        exporter_config: ExporterConfig,
    ) -> Self
    {
        let mut options = eventheader_dynamic::Provider::new_options();
        options = *options.group_name(provider_name);
        let mut provider: eventheader_dynamic::Provider = eventheader_dynamic::Provider::new(provider_name, &options);
        provider.register_set(eventheader::Level::Informational, exporter_config.get_log_event_keyword());
        provider.register_set(eventheader::Level::Verbose, exporter_config.get_log_event_keyword());
        provider.register_set(eventheader::Level::Warning, exporter_config.get_log_event_keyword());
        provider.register_set(eventheader::Level::Error, exporter_config.get_log_event_keyword());
        provider.register_set(eventheader::Level::CriticalError, exporter_config.get_log_event_keyword());


        provider.create_unregistered(true,eventheader::Level::Informational,exporter_config.get_log_event_keyword());
        provider.create_unregistered(true, eventheader::Level::Verbose, exporter_config.get_log_event_keyword());
        provider.create_unregistered(true, eventheader::Level::Warning, exporter_config.get_log_event_keyword());
        provider.create_unregistered(true, eventheader::Level::Error, exporter_config.get_log_event_keyword());
        provider.create_unregistered(true, eventheader::Level::CriticalError, exporter_config.get_log_event_keyword());


        UserEventsExporter { 
            provider: Arc::new(provider),
            exporter_config 
        }
    }

    fn add_attributes_to_event(
        &self,
        eb: &mut EventBuilder,
        attribs: &mut dyn Iterator<Item = (&Key, &AnyValue)>,
    ) {
        for attrib in attribs {
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
                },
                _ => ()
            }
        }
    }

    fn enabled(&self, level: u8, keyword: u64) -> bool{
        let es = self.provider.find_set(level.into(), keyword);
        if es.is_some() {
            es.unwrap().enabled()
        } else {
            false
        }
    }

    pub(crate) fn export_log_data(&self,log_data: &opentelemetry_sdk::export::logs::LogData ) -> opentelemetry_sdk::export::logs::ExportResult 
    {
        let level: Level = match log_data.record.severity_number.unwrap() {
            Severity::Debug 
            | Severity::Debug2 
            | Severity::Debug3 
            | Severity::Debug4
            | Severity::Trace
            | Severity::Trace2
            | Severity::Trace3
            | Severity::Trace4 => eventheader::Level::Verbose,

            Severity::Info
            | Severity::Info2
            | Severity::Info3
            | Severity::Info4 => eventheader::Level::Informational,

            Severity::Error
            | Severity::Error2
            | Severity::Error3
            | Severity::Error4 => eventheader::Level::Error,

            Severity::Fatal
            | Severity::Fatal2
            | Severity::Fatal3
            | Severity::Fatal4 => eventheader::Level::CriticalError,

            Severity::Warn
            | Severity::Warn2
            | Severity::Warn3
            | Severity::Warn4 => eventheader::Level::Warning
        };
        let log_es = if let Some(es) = self.provider.find_set(level.as_int().into(), self.exporter_config.get_log_event_keyword())
        {
            es
        } else {
            return Ok(());
        };
        if log_es.enabled() {
            print!("Enabled...\n");
            EBW.with(|eb| {

                let mut eb = eb.borrow_mut();
                //let attributes = log_data.resource.iter().chain(log_data.record.attributes.iter());
                let event_tags: u32 = 0; // TODO
                eb.reset(log_data.instrumentation.name.as_ref(), event_tags as u16);
                eb.opcode(Opcode::Info);

                eb.add_value("__csver__", 0x0401u16, FieldFormat::HexInt, 0);
                eb.add_struct("PartA", 1 /* + exts.len() as u8*/, 0);
                {
                    if (log_data.record.timestamp.is_some()) {
                        let time: String = chrono::DateTime::to_rfc3339(
                         &chrono::DateTime::<chrono::Utc>::from(log_data.record.timestamp.unwrap()));

                        eb.add_str("time", time, FieldFormat::Default, 0);
                    }
                }
                // Get Event_Id and Event_Name if present.
                let (mut event_id, mut event_name) = (0, "");
                let mut event_count = 0;
                for (k, v ) in log_data.record.attributes.as_ref().unwrap().into_iter() {
                    if k.as_str()== "event_id" {
                        event_id = match v {
                            AnyValue::Int(value) => {event_count = event_count + 1; *value}
                            _ => 0
                        }
                    }
                    if k.as_str() == "event_name" {
                        event_name = match v {
                            AnyValue::String(value) => {event_count = event_count + 1; value.as_ref()}
                            _ => ""
                        }
                    }
                }
                eb.add_struct("PartB", 2 + event_count, 0);
                {
                    eb.add_value("severityNumber", level.as_int(), FieldFormat::SignedInt, 0);
                    eb.add_str("body",
                        match log_data.record.body.as_ref().unwrap() {
                            AnyValue::Int(value) => value.to_string(),
                            AnyValue::String(value) => value.to_string(),
                            AnyValue::Boolean(value) => value.to_string(),
                            AnyValue::Double(value) => value.to_string(),
                            AnyValue::Bytes(value) => String::from_utf8_lossy(&value).to_string(),
                            AnyValue::ListAny(value) => "".to_string(),
                            AnyValue::Map(value) => "".to_string()
                        },
                        FieldFormat::Default,
                        0);
                    if event_id > 0 {
                        eb.add_value("eventId", event_id, FieldFormat::SignedInt, 0);
                    }
                    if event_name.len() > 0 {
                        eb.add_str("name", event_name, FieldFormat::Default, 0);
                    }
                };
                eb.add_struct("PartC", log_data.record.attributes.as_ref().unwrap().len() as u8, 0);
                {
                    self.add_attributes_to_event(&mut eb, &mut log_data.record.attributes.as_ref().unwrap().iter());
                }

                eb.write(&log_es, None, None);

                //TBD - Add remaining LogRecord attributes.
            });
            return Ok(());
        }
        print!("Not enabled...\n");
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
    async fn export(&mut self, batch: Vec<opentelemetry_sdk::export::logs::LogData>) -> opentelemetry_api::logs::LogResult<()> {
        for log_data in batch {
            let _ = self.export_log_data(&log_data);    
        }
        Ok(())
    } 
}
