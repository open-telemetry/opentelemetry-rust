use std::borrow::Cow;

use crate::{logs::LogRecord, KeyValue};

pub trait Logger {
    fn emit(&self, record: LogRecord);
}

pub trait LoggerProvider {
    type Logger: Logger;

    fn versioned_logger(
        &self,
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
        include_trace_context: bool,
    ) -> Self::Logger;

    fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger {
        self.versioned_logger(name, None, None, None, true)
    }
}
