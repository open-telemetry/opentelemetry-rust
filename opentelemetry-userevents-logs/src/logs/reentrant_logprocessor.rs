#![allow(unused_imports, unused_mut, unused_variables, unused_must_use, dead_code)]

use std::{sync::{Arc, Weak}, borrow::Cow};
use std::fmt::Debug;

use opentelemetry_api::{InstrumentationLibrary, logs::LogRecord, logs::LogResult};
use opentelemetry_sdk::{export::logs::LogExporter, export::logs::LogData, export::logs::ExportResult};

use crate::logs::exporter::*;
use crate::logs::exporter::ExporterConfig;

use std::cell::Cell; 

/// This export processor exports without synchronization.
/// This is currently only used in users_event exporter, where we know
/// that the underlying exporter is safe under concurrent calls

#[derive(Debug)]
pub struct ReentrantLogProcessor {
    event_exporter: UserEventsExporter,
}

impl ReentrantLogProcessor{
    /// constructor
    pub fn new(
        provider_name: &str,
        provider_group: ProviderGroup,
        exporter_config: ExporterConfig
    ) -> Self{
        let exporter = UserEventsExporter::new(
            provider_name, provider_group, exporter_config
        );
        ReentrantLogProcessor { event_exporter:  exporter }
    }
}

impl opentelemetry_sdk::logs::LogProcessor for ReentrantLogProcessor {

    fn emit(&self, data: LogData) {
        self.event_exporter.export_log_data(&data);
    }

    // This is a no-op as this processor doesn't keep anything 
    // in memory to be flushed out.
    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    // This is a no-op no special cleanup is required before
    // shutdown.
    fn shutdown(&mut self) -> LogResult<()>{
        Ok(())
    }
}