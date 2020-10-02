use opentelemetry::{api, exporter::trace};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub(crate) mod annotation;
pub(crate) mod endpoint;
pub(crate) mod span;

use endpoint::Endpoint;

/// Converts `api::Event` into an `annotation::Annotation`
impl Into<annotation::Annotation> for api::Event {
    fn into(self) -> annotation::Annotation {
        let timestamp = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as u64;

        annotation::Annotation::builder()
            .timestamp(timestamp)
            .value(self.name)
            .build()
    }
}

/// Converts StatusCode to str
fn from_statuscode_to_str(status_code: api::StatusCode) -> &'static str {
    match status_code {
        api::StatusCode::OK => "OK",
        api::StatusCode::Canceled => "CANCELLED",
        api::StatusCode::Unknown => "UNKNOWN",
        api::StatusCode::InvalidArgument => "INVALID_ARGUMENT",
        api::StatusCode::DeadlineExceeded => "DEADLINE_EXCEEDED",
        api::StatusCode::NotFound => "NOT_FOUND",
        api::StatusCode::AlreadyExists => "ALREADY_EXISTS",
        api::StatusCode::PermissionDenied => "PERMISSION_DENIED",
        api::StatusCode::ResourceExhausted => "RESOURSE_EXHAUSTED",
        api::StatusCode::FailedPrecondition => "FAILED_PRECONDITION",
        api::StatusCode::Aborted => "ABORTED",
        api::StatusCode::OutOfRange => "OUT_OF_RANGE",
        api::StatusCode::Unimplemented => "UNINPLEMENTED",
        api::StatusCode::Internal => "INTERNAL",
        api::StatusCode::Unavailable => "UNAVAILABLE",
        api::StatusCode::DataLoss => "DATA_LOSS",
        api::StatusCode::Unauthenticated => "UNAUTHENTICATED",
    }
}

/// Converts `api::SpanKind` into an `Option<span::Kind>`
fn into_zipkin_span_kind(kind: api::SpanKind) -> Option<span::Kind> {
    match kind {
        api::SpanKind::Client => Some(span::Kind::Client),
        api::SpanKind::Server => Some(span::Kind::Server),
        api::SpanKind::Producer => Some(span::Kind::Producer),
        api::SpanKind::Consumer => Some(span::Kind::Consumer),
        api::SpanKind::Internal => None,
    }
}

/// Converts a `trace::SpanData` to a `span::SpanData` for a given `ExporterConfig`, which can then
/// be ingested into a Zipkin collector.
pub(crate) fn into_zipkin_span(
    local_endpoint: Endpoint,
    span_data: &Arc<trace::SpanData>,
) -> span::Span {
    let mut user_defined_span_kind = false;
    let mut tags = map_from_kvs(
        span_data
            .attributes
            .iter()
            .map(|(k, v)| {
                if k == &api::Key::new("span.kind") {
                    user_defined_span_kind = true;
                }
                api::KeyValue::new(k.clone(), v.clone())
            })
            .chain(
                span_data
                    .resource
                    .iter()
                    .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone())),
            ),
    );

    tags.insert(
        "otel.status_code".into(),
        from_statuscode_to_str(span_data.status_code.clone()).into(),
    );
    tags.insert(
        "otel.status_description".into(),
        span_data.status_message.clone(),
    );

    span::Span::builder()
        .trace_id(span_data.span_context.trace_id().to_hex())
        .parent_id(span_data.parent_span_id.to_hex())
        .id(span_data.span_context.span_id().to_hex())
        .name(span_data.name.clone())
        .kind(if user_defined_span_kind {
            None
        } else {
            into_zipkin_span_kind(span_data.span_kind.clone())
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
        .annotations(
            span_data
                .message_events
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
        )
        .tags(tags)
        .build()
}

fn map_from_kvs<T>(kvs: T) -> HashMap<String, String>
where
    T: IntoIterator<Item = api::KeyValue>,
{
    let mut map: HashMap<String, String> = HashMap::new();
    for kv in kvs {
        map.insert(kv.key.into(), kv.value.into());
    }
    map
}
