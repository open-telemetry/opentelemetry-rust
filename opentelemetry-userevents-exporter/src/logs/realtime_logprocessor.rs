#![allow(unused_imports, unused_mut, unused_variables, unused_must_use, dead_code)]

use std::{sync::{Arc, Weak}, borrow::Cow};
use std::fmt::Debug;

use opentelemetry_api::{InstrumentationLibrary, logs::LogRecord, logs::LogResult};
use opentelemetry_sdk::{export::logs::LogExporter, export::logs::LogData, export::logs::ExportResult};

use crate::logs::exporter::*;
use crate::logs::exporter::ExporterConfig;
//use crate::exporter_traits::*;

use std::cell::Cell; 

#[derive(Debug)]
pub struct RealTimeLogProcessor {
    event_exporter: UserEventsExporter,
}

impl RealTimeLogProcessor{
    pub fn new(
        provider_name: &str,
        provider_group: ProviderGroup,
        exporter_config: ExporterConfig
    ) -> Self{
        let exporter = UserEventsExporter::new(
            provider_name, provider_group, exporter_config
        );
        RealTimeLogProcessor { event_exporter:  exporter }
    }
}

impl opentelemetry_sdk::logs::LogProcessor for RealTimeLogProcessor {

    fn emit(&self, data: LogData) {
        self.event_exporter.export_log_data(&data);
    }

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> LogResult<()>{
        Ok(())
    }
}