use std::fmt::Debug;

use opentelemetry::logs::LogResult;
use opentelemetry_sdk::export::logs::LogData;

#[cfg(feature = "logs_level_enabled")]
use opentelemetry_sdk::export::logs::LogExporter;

use crate::logs::exporter::ExporterConfig;
use crate::logs::exporter::*;

/// This export processor exports without synchronization.
/// This is currently only used in users_event exporter, where we know
/// that the underlying exporter is safe under concurrent calls

#[derive(Debug)]
pub struct ReentrantLogProcessor {
    event_exporter: UserEventsExporter,
}

impl ReentrantLogProcessor {
    /// constructor
    pub fn new(
        provider_name: &str,
        provider_group: ProviderGroup,
        exporter_config: ExporterConfig,
    ) -> Self {
        let exporter = UserEventsExporter::new(provider_name, provider_group, exporter_config);
        ReentrantLogProcessor {
            event_exporter: exporter,
        }
    }
}

impl opentelemetry_sdk::logs::LogProcessor for ReentrantLogProcessor {
    fn emit(&self, data: LogData) {
        _ = self.event_exporter.export_log_data(&data);
    }

    // This is a no-op as this processor doesn't keep anything
    // in memory to be flushed out.
    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    // This is a no-op no special cleanup is required before
    // shutdown.
    fn shutdown(&mut self) -> LogResult<()> {
        Ok(())
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(
        &self,
        level: opentelemetry::logs::Severity,
        target: &str,
        name: &str,
    ) -> bool {
        self.event_exporter.event_enabled(level, target, name)
    }
}
