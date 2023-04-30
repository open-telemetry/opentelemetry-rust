use std::borrow::Cow;

use crate::{logs::LogRecord, KeyValue};

pub trait Logger {
    fn emit(&self, record: LogRecord);
}

pub trait LoggerProvider {
    type Logger: Logger;

    fn versioned_logger(
        &self,
        name: Cow<'static, str>,
        version: Option<Cow<'static, str>>,
        schema_url: Option<Cow<'static, str>>,
        attributes: Option<Vec<KeyValue>>,
    ) -> Self::Logger;

    fn logger(&self, name: Cow<'static, str>) -> Self::Logger {
        self.versioned_logger(name, None, None, None)
    }
}
