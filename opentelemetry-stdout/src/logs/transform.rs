use std::{borrow::Cow, collections::HashMap, time::SystemTime};

use crate::common::{
    as_human_readable, as_opt_human_readable, as_opt_unix_nano, as_unix_nano, KeyValue, Resource,
    Scope, Value,
};
use opentelemetry::AttributeSet;
use serde::Serialize;

/// Transformed logs data that can be serialized.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogData {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    resource_logs: Vec<ResourceLogs>,
}

impl From<Vec<opentelemetry_sdk::export::logs::LogData>> for LogData {
    fn from(sdk_logs: Vec<opentelemetry_sdk::export::logs::LogData>) -> LogData {
        let mut resource_logs = HashMap::<AttributeSet, ResourceLogs>::new();

        for sdk_log in sdk_logs {
            let resource_schema_url = sdk_log.resource.schema_url().map(|s| s.to_string().into());
            let schema_url = sdk_log.instrumentation.schema_url.clone();
            let scope: Scope = sdk_log.instrumentation.clone().into();
            let resource: Resource = sdk_log.resource.as_ref().into();

            let rl = resource_logs
                .entry(sdk_log.resource.as_ref().to_attribute_set())
                .or_insert_with(move || ResourceLogs {
                    resource,
                    scope_logs: Vec::with_capacity(1),
                    schema_url: resource_schema_url,
                });

            match rl.scope_logs.iter_mut().find(|sl| sl.scope == scope) {
                Some(sl) => sl.log_records.push(sdk_log.into()),
                None => rl.scope_logs.push(ScopeLogs {
                    scope,
                    log_records: vec![sdk_log.into()],
                    schema_url,
                }),
            }
        }

        LogData {
            resource_logs: resource_logs.into_values().collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceLogs {
    resource: Resource,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    scope_logs: Vec<ScopeLogs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<Cow<'static, str>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopeLogs {
    scope: Scope,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    log_records: Vec<LogRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<Cow<'static, str>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogRecord {
    #[serde(serialize_with = "as_opt_unix_nano")]
    time_unix_nano: Option<SystemTime>,
    #[serde(serialize_with = "as_opt_human_readable")]
    time: Option<SystemTime>,
    #[serde(serialize_with = "as_unix_nano")]
    observed_time_unix_nano: SystemTime,
    #[serde(serialize_with = "as_human_readable")]
    observed_time: SystemTime,
    severity_number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity_text: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
}

impl From<opentelemetry_sdk::export::logs::LogData> for LogRecord {
    fn from(value: opentelemetry_sdk::export::logs::LogData) -> Self {
        LogRecord {
            trace_id: value
                .record
                .trace_context
                .as_ref()
                .map(|c| c.trace_id.to_string()),
            span_id: value
                .record
                .trace_context
                .as_ref()
                .map(|c| c.span_id.to_string()),
            flags: value
                .record
                .trace_context
                .map(|c| c.trace_flags.map(|f| f.to_u8()))
                .unwrap_or_default(),
            time_unix_nano: value.record.timestamp,
            time: value.record.timestamp,
            observed_time_unix_nano: value.record.observed_timestamp,
            observed_time: value.record.observed_timestamp,
            severity_number: value
                .record
                .severity_number
                .map(|u| u as u32)
                .unwrap_or_default(),
            attributes: value
                .record
                .attributes
                .map(|attrs| {
                    attrs
                        .into_iter()
                        .map(|(key, value)| (key, value).into())
                        .collect()
                })
                .unwrap_or_default(),
            dropped_attributes_count: 0,
            severity_text: value.record.severity_text,
            body: value.record.body.map(|a| a.into()),
        }
    }
}
