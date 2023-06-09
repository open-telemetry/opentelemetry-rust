#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

use std::borrow::Cow;
pub type ProviderGroup = Option<Cow<'static, str>>;

pub trait KeywordLevelProvider: Send + Sync {

    /// The keyword(s) to use for Log events.
    fn get_log_event_keywords(&self) -> u64;

    /// The level to use for Log events.
    fn get_log_event_level(&self) -> u8;
}

pub(crate) struct ExporterConfig<T: KeywordLevelProvider> {
    pub(crate) kwl: T,
}

pub(crate) struct DefaultKeywordLevelProvider;


impl KeywordLevelProvider for DefaultKeywordLevelProvider {
    #[inline(always)]
    fn get_log_event_keywords(&self) -> u64 {
        0x1000
    }

    #[inline(always)]
    fn get_log_event_level(&self) -> u8 {
        4 // Level::Informational
    }
}

impl KeywordLevelProvider for Box<dyn KeywordLevelProvider> {

    #[inline(always)]
    fn get_log_event_keywords(&self) -> u64 {
        self.as_ref().get_log_event_keywords()
    }

    #[inline(always)]
    fn get_log_event_level(&self) -> u8 {
        self.as_ref().get_log_event_level()
    }
}

impl<T: KeywordLevelProvider> KeywordLevelProvider for ExporterConfig<T> {
    #[inline(always)]
    fn get_log_event_keywords(&self) -> u64 {
        self.kwl.get_log_event_keywords()
    }

    #[inline(always)]
    fn get_log_event_level(&self) -> u8 {
        self.kwl.get_log_event_level()
    }
}

pub trait EventExporter {
    fn enabled(&self, level: u8, keyword: u64) -> bool;
    fn export_log_data (
        &self,
        log_data: &opentelemetry_sdk::export::logs::LogData,
    ) -> opentelemetry_sdk::export::logs::ExportResult;
}

