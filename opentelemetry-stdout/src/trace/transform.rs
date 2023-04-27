use std::{borrow::Cow, collections::HashMap, time::SystemTime};

use opentelemetry_sdk::AttributeSet;
use serde::{Serialize, Serializer};

use crate::common::{as_unix_nano, KeyValue, Resource, Scope};

/// Transformed trace data that can be serialized
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanData {
    resource_spans: Vec<ResourceSpans>,
}

impl From<Vec<opentelemetry_sdk::export::trace::SpanData>> for SpanData {
    fn from(sdk_spans: Vec<opentelemetry_sdk::export::trace::SpanData>) -> Self {
        let mut resource_spans = HashMap::<AttributeSet, ResourceSpans>::new();
        for sdk_span in sdk_spans {
            let resource_schema_url = sdk_span.resource.schema_url().map(|s| s.to_string().into());
            let schema_url = sdk_span.instrumentation_lib.schema_url.clone();
            let scope = sdk_span.instrumentation_lib.clone().into();
            let resource = sdk_span.resource.as_ref().into();

            let rs = resource_spans
                .entry(sdk_span.resource.as_ref().into())
                .or_insert_with(move || ResourceSpans {
                    resource,
                    scope_spans: Vec::with_capacity(1),
                    schema_url: resource_schema_url,
                });

            match rs.scope_spans.iter_mut().find(|ss| ss.scope == scope) {
                Some(ss) => ss.spans.push(sdk_span.into()),
                None => rs.scope_spans.push(ScopeSpans {
                    scope,
                    spans: vec![sdk_span.into()],
                    schema_url,
                }),
            };
        }

        SpanData {
            resource_spans: resource_spans.into_values().collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceSpans {
    resource: Resource,
    scope_spans: Vec<ScopeSpans>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<Cow<'static, str>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopeSpans {
    scope: Scope,
    spans: Vec<Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<Cow<'static, str>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Span {
    trace_id: String,
    span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_state: Option<String>,
    parent_span_id: String,
    name: Cow<'static, str>,
    kind: SpanKind,
    #[serde(serialize_with = "as_unix_nano")]
    start_time_unix_nano: SystemTime,
    #[serde(serialize_with = "as_unix_nano")]
    end_time_unix_nano: SystemTime,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    events: Vec<Event>,
    dropped_events_count: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<Link>,
    dropped_links_count: u32,
    status: Status,
}

impl From<opentelemetry_sdk::export::trace::SpanData> for Span {
    fn from(value: opentelemetry_sdk::export::trace::SpanData) -> Self {
        Span {
            trace_id: format!("{:x}", value.span_context.trace_id()),
            span_id: format!("{:x}", value.span_context.span_id()),
            trace_state: Some(value.span_context.trace_state().header()).filter(|s| !s.is_empty()),
            parent_span_id: Some(format!("{:x}", value.parent_span_id))
                .filter(|s| s != "0")
                .unwrap_or_default(),
            name: value.name,
            kind: value.span_kind.into(),
            start_time_unix_nano: value.start_time,
            end_time_unix_nano: value.end_time,
            dropped_attributes_count: value.attributes.dropped_count(),
            attributes: value.attributes.into_iter().map(Into::into).collect(),
            dropped_events_count: value.events.dropped_count(),
            events: value.events.into_iter().map(Into::into).collect(),
            dropped_links_count: value.links.dropped_count(),
            links: value.links.into_iter().map(Into::into).collect(),
            status: value.status.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SpanKind {
    #[allow(dead_code)]
    Unspecified = 0,
    Internal = 1,
    Server = 2,
    Client = 3,
    Producer = 4,
    Consumer = 5,
}

impl Serialize for SpanKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u32 as u8)
    }
}

impl From<opentelemetry_api::trace::SpanKind> for SpanKind {
    fn from(value: opentelemetry_api::trace::SpanKind) -> Self {
        match value {
            opentelemetry_api::trace::SpanKind::Client => SpanKind::Client,
            opentelemetry_api::trace::SpanKind::Server => SpanKind::Server,
            opentelemetry_api::trace::SpanKind::Producer => SpanKind::Producer,
            opentelemetry_api::trace::SpanKind::Consumer => SpanKind::Consumer,
            opentelemetry_api::trace::SpanKind::Internal => SpanKind::Internal,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Event {
    name: Cow<'static, str>,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
}

impl From<opentelemetry_api::trace::Event> for Event {
    fn from(value: opentelemetry_api::trace::Event) -> Self {
        Event {
            name: value.name,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Link {
    trace_id: String,
    span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_state: Option<String>,
    attributes: Vec<KeyValue>,
    dropped_attributes_count: u32,
}

impl From<opentelemetry_api::trace::Link> for Link {
    fn from(value: opentelemetry_api::trace::Link) -> Self {
        Link {
            trace_id: format!("{:x}", value.span_context.trace_id()),
            span_id: format!("{:x}", value.span_context.span_id()),
            trace_state: Some(value.span_context.trace_state().header()).filter(|s| !s.is_empty()),
            attributes: value.attributes.into_iter().map(Into::into).collect(),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "is_zero")]
    code: u32,
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}

impl From<opentelemetry_api::trace::Status> for Status {
    fn from(value: opentelemetry_api::trace::Status) -> Self {
        match value {
            opentelemetry_api::trace::Status::Unset => Status {
                message: None,
                code: 0,
            },
            opentelemetry_api::trace::Status::Error { description } => Status {
                message: Some(description),
                code: 1,
            },
            opentelemetry_api::trace::Status::Ok => Status {
                message: None,
                code: 2,
            },
        }
    }
}
