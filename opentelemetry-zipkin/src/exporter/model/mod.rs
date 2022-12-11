use opentelemetry::{
    sdk::export::trace,
    trace::{SpanKind, Status},
    Key, KeyValue,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub(crate) mod annotation;
pub(crate) mod endpoint;
pub(crate) mod span;

use endpoint::Endpoint;

const INSTRUMENTATION_LIBRARY_NAME: &str = "otel.library.name";
const INSTRUMENTATION_LIBRARY_VERSION: &str = "otel.library.version";
const OTEL_ERROR_DESCRIPTION: &str = "error";
const OTEL_STATUS_CODE: &str = "otel.status_code";

/// Converts `SpanKind` into an `Option<span::Kind>`
fn into_zipkin_span_kind(kind: SpanKind) -> Option<span::Kind> {
    match kind {
        SpanKind::Client => Some(span::Kind::Client),
        SpanKind::Server => Some(span::Kind::Server),
        SpanKind::Producer => Some(span::Kind::Producer),
        SpanKind::Consumer => Some(span::Kind::Consumer),
        SpanKind::Internal => None,
    }
}

/// Converts a `trace::SpanData` to a `span::SpanData` for a given `ExporterConfig`, which can then
/// be ingested into a Zipkin collector.
pub(crate) fn into_zipkin_span(local_endpoint: Endpoint, span_data: trace::SpanData) -> span::Span {
    // see tests in create/exporter/model/span.rs
    let mut user_defined_span_kind = false;
    let mut tags = map_from_kvs(
        span_data
            .attributes
            .into_iter()
            .map(|(k, v)| {
                if k == Key::new("span.kind") {
                    user_defined_span_kind = true;
                }
                KeyValue::new(k, v)
            })
            .chain(
                [
                    (
                        INSTRUMENTATION_LIBRARY_NAME,
                        Some(span_data.instrumentation_lib.name),
                    ),
                    (
                        INSTRUMENTATION_LIBRARY_VERSION,
                        span_data.instrumentation_lib.version,
                    ),
                ]
                .into_iter()
                .filter_map(|(key, val)| val.map(|val| KeyValue::new(key, val))),
            )
            .filter(|kv| kv.key.as_str() != "error"),
    );

    match span_data.status {
        Status::Unset => {}
        Status::Ok => {
            tags.insert(OTEL_STATUS_CODE.into(), "OK".into());
        }
        Status::Error {
            description: message,
        } => {
            tags.insert(OTEL_STATUS_CODE.into(), "ERROR".into());
            tags.insert(OTEL_ERROR_DESCRIPTION.into(), message.into_owned());
        }
    };

    span::Span::builder()
        .trace_id(span_data.span_context.trace_id().to_string())
        .parent_id(span_data.parent_span_id.to_string())
        .id(span_data.span_context.span_id().to_string())
        .name(span_data.name.into_owned())
        .kind(if user_defined_span_kind {
            None
        } else {
            into_zipkin_span_kind(span_data.span_kind)
        })
        .timestamp(
            span_data
                .start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_micros() as u64,
        )
        .duration(
            span_data
                .end_time
                .duration_since(span_data.start_time)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_micros() as u64,
        )
        .local_endpoint(local_endpoint)
        .annotations(span_data.events.into_iter().map(Into::into).collect())
        .tags(tags)
        .build()
}

fn map_from_kvs<T>(kvs: T) -> HashMap<String, String>
where
    T: IntoIterator<Item = KeyValue>,
{
    let mut map: HashMap<String, String> = HashMap::new();
    for kv in kvs {
        map.insert(kv.key.into(), kv.value.to_string());
    }
    map
}
