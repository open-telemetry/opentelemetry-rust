use chrono::{prelude::*, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt;

pub enum Error {
    /// 400 Bad Request
    BadRequest,
    // ...
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TracingFmt<T> {
    pub when: NaiveDateTime,
    pub owner: String,
    pub params: String,
    pub content: T,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomError {
    pub code: u16,
    pub message: String,
    pub timestamp: NaiveDateTime,
}

impl CustomError {
    pub fn new(code: u16, message: String) -> CustomError {
        CustomError {
            code,
            message,
            timestamp: Utc::now().naive_utc(),
        }
    }

    /// Create custom error from Error. Ex: CustomError::from(Error::NotFound)
    pub fn from(err: Error) -> CustomError {
        match err {
            Error::BadRequest => CustomError::new(400, "Bad Request".to_owned()),
            // ...
        }
    }

    /// Returns a custom trace-friendly structure for logging
    pub fn tracing_fmt<T>(owner: String, params: String, content: T) -> TracingFmt<T>
    where
        T: Serialize,
    {
        TracingFmt {
            when: Utc::now().naive_utc(),
            owner,
            params,
            content,
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}
