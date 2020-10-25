use opentelemetry::{api, exporter::trace};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub(crate) mod annotation;
pub(crate) mod endpoint;
pub(crate) mod span;

use endpoint::Endpoint;

/// Instrument Library name MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_NAME: &str = "otel.library.name";

/// Instrument Library version MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_VERSION: &str = "otel.library.version";

/// Converts `api::trace::Event` into an `annotation::Annotation`
impl Into<annotation::Annotation> for api::trace::Event {
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
fn from_statuscode_to_str(status_code: api::trace::StatusCode) -> &'static str {
    match status_code {
        api::trace::StatusCode::OK => "OK",
        api::trace::StatusCode::Canceled => "CANCELLED",
        api::trace::StatusCode::Unknown => "UNKNOWN",
        api::trace::StatusCode::InvalidArgument => "INVALID_ARGUMENT",
        api::trace::StatusCode::DeadlineExceeded => "DEADLINE_EXCEEDED",
        api::trace::StatusCode::NotFound => "NOT_FOUND",
        api::trace::StatusCode::AlreadyExists => "ALREADY_EXISTS",
        api::trace::StatusCode::PermissionDenied => "PERMISSION_DENIED",
        api::trace::StatusCode::ResourceExhausted => "RESOURSE_EXHAUSTED",
        api::trace::StatusCode::FailedPrecondition => "FAILED_PRECONDITION",
        api::trace::StatusCode::Aborted => "ABORTED",
        api::trace::StatusCode::OutOfRange => "OUT_OF_RANGE",
        api::trace::StatusCode::Unimplemented => "UNINPLEMENTED",
        api::trace::StatusCode::Internal => "INTERNAL",
        api::trace::StatusCode::Unavailable => "UNAVAILABLE",
        api::trace::StatusCode::DataLoss => "DATA_LOSS",
        api::trace::StatusCode::Unauthenticated => "UNAUTHENTICATED",
    }
}

/// Converts `api::trace::SpanKind` into an `Option<span::Kind>`
fn into_zipkin_span_kind(kind: api::trace::SpanKind) -> Option<span::Kind> {
    match kind {
        api::trace::SpanKind::Client => Some(span::Kind::Client),
        api::trace::SpanKind::Server => Some(span::Kind::Server),
        api::trace::SpanKind::Producer => Some(span::Kind::Producer),
        api::trace::SpanKind::Consumer => Some(span::Kind::Consumer),
        api::trace::SpanKind::Internal => None,
    }
}

/// Converts a `trace::SpanData` to a `span::SpanData` for a given `ExporterConfig`, which can then
/// be ingested into a Zipkin collector.
pub(crate) fn into_zipkin_span(local_endpoint: Endpoint, span_data: trace::SpanData) -> span::Span {
    let mut user_defined_span_kind = false;
    let mut tags = map_from_kvs(
        span_data
            .attributes
            .into_iter()
            .map(|(k, v)| {
                if k == api::Key::new("span.kind") {
                    user_defined_span_kind = true;
                }
                api::KeyValue::new(k, v)
            })
            .chain(
                span_data
                    .resource
                    .iter()
                    .map(|(k, v)| api::KeyValue::new(k.clone(), v.clone())),
            )
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
                .iter()
                .filter_map(|(key, val)| val.map(|val| api::KeyValue::new(*key, val))),
            ),
    );

    tags.insert(
        "otel.status_code".into(),
        from_statuscode_to_str(span_data.status_code).into(),
    );
    tags.insert("otel.status_description".into(), span_data.status_message);

    span::Span::builder()
        .trace_id(span_data.span_context.trace_id().to_hex())
        .parent_id(span_data.parent_span_id.to_hex())
        .id(span_data.span_context.span_id().to_hex())
        .name(span_data.name)
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
        .annotations(
            span_data
                .message_events
                .into_iter()
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
