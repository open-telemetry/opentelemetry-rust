//! # Jaeger Exporter
//!
mod agent;
#[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
mod collector;
pub(crate) mod runtime;
#[allow(clippy::all, unreachable_pub, dead_code)]
#[rustfmt::skip] // don't format generated files
mod thrift;
pub mod config;
pub(crate) mod transport;
mod uploader;

// Linting isn't detecting that it's used seems like linting bug.
#[allow(unused_imports)]
#[cfg(feature = "surf_collector_client")]
use std::convert::TryFrom;

use self::runtime::JaegerTraceRuntime;
use self::thrift::jaeger;
use async_trait::async_trait;
use std::convert::TryInto;

#[cfg(feature = "isahc_collector_client")]
#[allow(unused_imports)] // this is actually used to configure authentication
use isahc::prelude::Configurable;

use opentelemetry::sdk::export::ExportError;
use opentelemetry::{
    sdk,
    sdk::export::trace,
    trace::{Event, Link, SpanKind, StatusCode},
    Key, KeyValue,
};

use crate::exporter::uploader::Uploader;
use std::time::{Duration, SystemTime};

/// Instrument Library name MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_NAME: &str = "otel.library.name";

/// Instrument Library version MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_VERSION: &str = "otel.library.version";

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    process: jaeger::Process,
    /// Whether or not to export instrumentation information.
    export_instrumentation_lib: bool,
    uploader: Box<dyn Uploader>,
}

impl Exporter {
    fn new(
        process: jaeger::Process,
        export_instrumentation_lib: bool,
        uploader: Box<dyn Uploader>,
    ) -> Exporter {
        Exporter {
            process,
            export_instrumentation_lib,
            uploader,
        }
    }
}

/// Jaeger process configuration
#[derive(Debug, Default)]
pub struct Process {
    /// Jaeger service name
    pub service_name: String,
    /// Jaeger tags
    pub tags: Vec<KeyValue>,
}

#[async_trait]
impl trace::SpanExporter for Exporter {
    /// Export spans to Jaeger
    async fn export(&mut self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
        let mut jaeger_spans: Vec<jaeger::Span> = Vec::with_capacity(batch.len());
        let process = self.process.clone();

        for span in batch.into_iter() {
            jaeger_spans.push(convert_otel_span_into_jaeger_span(
                span,
                self.export_instrumentation_lib,
            ));
        }

        self.uploader
            .upload(jaeger::Batch::new(process, jaeger_spans))
            .await
    }
}

fn links_to_references(links: sdk::trace::EvictedQueue<Link>) -> Option<Vec<jaeger::SpanRef>> {
    if !links.is_empty() {
        let refs = links
            .iter()
            .map(|link| {
                let span_context = link.span_context();
                let trace_id_bytes = span_context.trace_id().to_bytes();
                let (high, low) = trace_id_bytes.split_at(8);
                let trace_id_high = i64::from_be_bytes(high.try_into().unwrap());
                let trace_id_low = i64::from_be_bytes(low.try_into().unwrap());

                jaeger::SpanRef::new(
                    jaeger::SpanRefType::FollowsFrom,
                    trace_id_low,
                    trace_id_high,
                    i64::from_be_bytes(span_context.span_id().to_bytes()),
                )
            })
            .collect();
        Some(refs)
    } else {
        None
    }
}

/// Convert spans to jaeger thrift span for exporting.
fn convert_otel_span_into_jaeger_span(
    span: trace::SpanData,
    export_instrument_lib: bool,
) -> jaeger::Span {
    let trace_id_bytes = span.span_context.trace_id().to_bytes();
    let (high, low) = trace_id_bytes.split_at(8);
    let trace_id_high = i64::from_be_bytes(high.try_into().unwrap());
    let trace_id_low = i64::from_be_bytes(low.try_into().unwrap());
    jaeger::Span {
        trace_id_low,
        trace_id_high,
        span_id: i64::from_be_bytes(span.span_context.span_id().to_bytes()),
        parent_span_id: i64::from_be_bytes(span.parent_span_id.to_bytes()),
        operation_name: span.name.into_owned(),
        references: links_to_references(span.links),
        flags: span.span_context.trace_flags().to_u8() as i32,
        start_time: span
            .start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64,
        duration: span
            .end_time
            .duration_since(span.start_time)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as i64,
        tags: Some(build_span_tags(
            span.attributes,
            if export_instrument_lib {
                Some(span.instrumentation_lib)
            } else {
                None
            },
            span.status_code,
            span.status_message.into_owned(),
            span.span_kind,
        )),
        logs: events_to_logs(span.events),
    }
}

fn build_span_tags(
    attrs: sdk::trace::EvictedHashMap,
    instrumentation_lib: Option<sdk::InstrumentationLibrary>,
    status_code: StatusCode,
    status_description: String,
    kind: SpanKind,
) -> Vec<jaeger::Tag> {
    let mut user_overrides = UserOverrides::default();
    // TODO determine if namespacing is required to avoid collisions with set attributes
    let mut tags = attrs
        .into_iter()
        .map(|(k, v)| {
            user_overrides.record_attr(k.as_str());
            KeyValue::new(k, v).into()
        })
        .collect::<Vec<_>>();

    if let Some(instrumentation_lib) = instrumentation_lib {
        // Set instrument library tags
        tags.push(KeyValue::new(INSTRUMENTATION_LIBRARY_NAME, instrumentation_lib.name).into());
        if let Some(version) = instrumentation_lib.version {
            tags.push(KeyValue::new(INSTRUMENTATION_LIBRARY_VERSION, version).into())
        }
    }

    if !user_overrides.span_kind && kind != SpanKind::Internal {
        tags.push(Key::new(SPAN_KIND).string(kind.to_string()).into());
    }

    if status_code != StatusCode::Unset {
        // Ensure error status is set unless user has already overrided it
        if status_code == StatusCode::Error && !user_overrides.error {
            tags.push(Key::new(ERROR).bool(true).into());
        }
        if !user_overrides.status_code {
            if status_code == StatusCode::Ok {
                tags.push(KeyValue::new(OTEL_STATUS_CODE, "OK").into());
            } else if status_code == StatusCode::Error {
                tags.push(KeyValue::new(OTEL_STATUS_CODE, "ERROR").into());
            }
        }
        // set status message if there is one
        if !status_description.is_empty() && !user_overrides.status_description {
            tags.push(
                Key::new(OTEL_STATUS_DESCRIPTION)
                    .string(status_description)
                    .into(),
            );
        }
    }

    tags
}

const ERROR: &str = "error";
const SPAN_KIND: &str = "span.kind";
const OTEL_STATUS_CODE: &str = "otel.status_code";
const OTEL_STATUS_DESCRIPTION: &str = "otel.status_description";

#[derive(Default)]
struct UserOverrides {
    error: bool,
    span_kind: bool,
    status_code: bool,
    status_description: bool,
}

impl UserOverrides {
    fn record_attr(&mut self, attr: &str) {
        match attr {
            ERROR => self.error = true,
            SPAN_KIND => self.span_kind = true,
            OTEL_STATUS_CODE => self.status_code = true,
            OTEL_STATUS_DESCRIPTION => self.status_description = true,
            _ => (),
        }
    }
}

fn events_to_logs(events: sdk::trace::EvictedQueue<Event>) -> Option<Vec<jaeger::Log>> {
    if events.is_empty() {
        None
    } else {
        Some(events.into_iter().map(Into::into).collect())
    }
}

/// Wrap type for errors from opentelemetry jaeger
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error from thrift agents.
    #[error("thrift agent failed with {0}")]
    ThriftAgentError(#[from] ::thrift::Error),
    /// No http client provided.
    #[cfg(feature = "collector_client")]
    #[error(
        "No http client provided. Consider enable one of the `surf_collector_client`, \
        `reqwest_collector_client`, `reqwest_blocking_collector_client`, `isahc_collector_client` \
        feature to have a default implementation. Or use with_http_client method in pipeline to \
        provide your own implementation."
    )]
    NoHttpClient,
    /// reqwest client errors
    #[error("reqwest failed with {0}")]
    #[cfg(any(
        feature = "reqwest_collector_client",
        feature = "reqwest_blocking_collector_client"
    ))]
    ReqwestClientError(#[from] reqwest::Error),

    /// invalid collector uri is provided.
    #[error("collector uri is invalid, {0}")]
    #[cfg(any(feature = "collector_client", feature = "wasm_collector_client"))]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Pipeline fails because one of the configurations is invalid.
    #[error("{pipeline_name} pipeline fails because one of the configuration, {config_name}, is invalid. {reason}")]
    ConfigError {
        /// the name of the pipeline. It can be `agent`, `collector` or `wasm collector`
        pipeline_name: &'static str,
        /// config name that has the error.
        config_name: &'static str,
        /// the underlying error message.
        reason: String,
    },
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "jaeger"
    }
}

#[cfg(test)]
mod tests {
    use super::SPAN_KIND;
    use crate::exporter::thrift::jaeger::Tag;
    use crate::exporter::{build_span_tags, OTEL_STATUS_CODE, OTEL_STATUS_DESCRIPTION};
    use opentelemetry::sdk::trace::EvictedHashMap;
    use opentelemetry::trace::{SpanKind, StatusCode};
    use opentelemetry::KeyValue;

    fn assert_tag_contains(tags: Vec<Tag>, key: &'static str, expect_val: &'static str) {
        assert_eq!(
            tags.into_iter()
                .filter(|tag| tag.key.as_str() == key
                    && tag.v_str.as_deref().unwrap_or("") == expect_val)
                .count(),
            1,
            "Expect a tag {} with value {} but found nothing",
            key,
            expect_val
        );
    }

    fn assert_tag_not_contains(tags: Vec<Tag>, key: &'static str) {
        assert_eq!(
            tags.into_iter()
                .filter(|tag| tag.key.as_str() == key)
                .count(),
            0,
            "Not expect tag {}, but found something",
            key
        );
    }

    fn get_error_tag_test_data() -> Vec<(
        StatusCode,
        String,
        Option<&'static str>,
        Option<&'static str>,
    )> {
        // StatusCode, error message, OTEL_STATUS_CODE tag value, OTEL_STATUS_DESCRIPTION tag value
        vec![
            (StatusCode::Error, "".into(), Some("ERROR"), None),
            (StatusCode::Unset, "".into(), None, None),
            // When status is ok, no description should be in span data. This should be ensured by Otel API
            (StatusCode::Ok, "".into(), Some("OK"), None),
            (
                StatusCode::Error,
                "have message".into(),
                Some("ERROR"),
                Some("have message"),
            ),
            (StatusCode::Unset, "have message".into(), None, None),
        ]
    }

    #[test]
    fn test_set_status() {
        for (status_code, error_msg, status_tag_val, msg_tag_val) in get_error_tag_test_data() {
            let tags = build_span_tags(
                EvictedHashMap::new(20, 20),
                None,
                status_code,
                error_msg,
                SpanKind::Client,
            );
            if let Some(val) = status_tag_val {
                assert_tag_contains(tags.clone(), OTEL_STATUS_CODE, val);
            } else {
                assert_tag_not_contains(tags.clone(), OTEL_STATUS_CODE);
            }

            if let Some(val) = msg_tag_val {
                assert_tag_contains(tags.clone(), OTEL_STATUS_DESCRIPTION, val);
            } else {
                assert_tag_not_contains(tags.clone(), OTEL_STATUS_DESCRIPTION);
            }
        }
    }

    #[test]
    fn ignores_user_set_values() {
        let mut attributes = EvictedHashMap::new(20, 20);
        let user_error = true;
        let user_kind = "server";
        let user_status_code = StatusCode::Error;
        let user_status_description = "Something bad happened";
        attributes.insert(KeyValue::new("error", user_error));
        attributes.insert(KeyValue::new(SPAN_KIND, user_kind));
        attributes.insert(KeyValue::new(OTEL_STATUS_CODE, "ERROR"));
        attributes.insert(KeyValue::new(
            OTEL_STATUS_DESCRIPTION,
            user_status_description,
        ));
        let tags = build_span_tags(
            attributes,
            None,
            user_status_code,
            user_status_description.to_string(),
            SpanKind::Client,
        );

        assert!(tags
            .iter()
            .filter(|tag| tag.key.as_str() == "error")
            .all(|tag| tag.v_bool.unwrap()));
        assert_tag_contains(tags.clone(), SPAN_KIND, user_kind);
        assert_tag_contains(tags.clone(), OTEL_STATUS_CODE, "ERROR");
        assert_tag_contains(tags, OTEL_STATUS_DESCRIPTION, user_status_description);
    }
}
