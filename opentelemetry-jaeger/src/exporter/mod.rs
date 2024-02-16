//! # Jaeger Exporter
//!
// Linting isn't detecting that it's used seems like linting bug.
#[allow(unused_imports)]
use std::convert::TryInto;
use std::fmt::Display;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use futures_core::future::BoxFuture;
#[cfg(feature = "isahc_collector_client")]
#[allow(unused_imports)] // this is actually used to configure authentication
use isahc::prelude::Configurable;

use opentelemetry::{
    trace::{Event, Link, SpanKind, Status},
    InstrumentationLibrary, Key, KeyValue,
};
use opentelemetry_sdk::export::{
    trace::{ExportResult, SpanData, SpanExporter},
    ExportError,
};
use opentelemetry_sdk::trace::SpanEvents;

use crate::exporter::uploader::Uploader;

use self::runtime::JaegerTraceRuntime;
use self::thrift::jaeger;

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

/// Instrument Library name MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_NAME: &str = "otel.library.name";

/// Instrument Library version MUST be reported in Jaeger Span tags with the following key
const INSTRUMENTATION_LIBRARY_VERSION: &str = "otel.library.version";

/// Jaeger span exporter
#[derive(Debug)]
pub struct Exporter {
    /// Whether or not to export instrumentation information.
    export_instrumentation_lib: bool,
    uploader: Arc<dyn Uploader>,
    process: jaeger::Process,
}

impl Exporter {
    fn new(
        process: jaeger::Process,
        export_instrumentation_lib: bool,
        uploader: Arc<dyn Uploader>,
    ) -> Exporter {
        Exporter {
            export_instrumentation_lib,
            uploader,
            process,
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

impl SpanExporter for Exporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let mut jaeger_spans: Vec<jaeger::Span> = Vec::with_capacity(batch.len());
        let process = self.process.clone();

        for span in batch.into_iter() {
            jaeger_spans.push(convert_otel_span_into_jaeger_span(
                span,
                self.export_instrumentation_lib,
            ));
        }

        let uploader = self.uploader.clone();
        Box::pin(async move {
            uploader
                .upload(jaeger::Batch::new(process, jaeger_spans))
                .await
        })
    }
}

fn links_to_references(links: &[Link]) -> Option<Vec<jaeger::SpanRef>> {
    if !links.is_empty() {
        let refs = links
            .iter()
            .map(|link| {
                let span_context = &link.span_context;
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
fn convert_otel_span_into_jaeger_span(span: SpanData, export_instrument_lib: bool) -> jaeger::Span {
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
        references: links_to_references(span.links.as_ref()),
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
            span.status,
            span.span_kind,
        )),
        logs: events_to_logs(span.events),
    }
}

fn build_span_tags(
    attrs: Vec<KeyValue>,
    instrumentation_lib: Option<InstrumentationLibrary>,
    status: Status,
    kind: SpanKind,
) -> Vec<jaeger::Tag> {
    let mut user_overrides = UserOverrides::default();
    // TODO determine if namespacing is required to avoid collisions with set attributes
    let mut tags = attrs
        .into_iter()
        .map(|kv| {
            user_overrides.record_attr(kv.key.as_str());
            kv.into()
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
        tags.push(Key::new(SPAN_KIND).string(format_span_kind(kind)).into());
    }

    match status {
        Status::Unset => {}
        Status::Ok => {
            if !user_overrides.status_code {
                tags.push(KeyValue::new(OTEL_STATUS_CODE, "OK").into());
            }
        }
        Status::Error {
            description: message,
        } => {
            if !user_overrides.error {
                tags.push(Key::new(ERROR).bool(true).into());
            }

            if !user_overrides.status_code {
                tags.push(KeyValue::new(OTEL_STATUS_CODE, "ERROR").into());
            }

            if !message.is_empty() && !user_overrides.status_description {
                tags.push(Key::new(OTEL_STATUS_DESCRIPTION).string(message).into());
            }
        }
    }

    tags
}

fn format_span_kind(kind: SpanKind) -> &'static str {
    match kind {
        SpanKind::Client => "client",
        SpanKind::Server => "server",
        SpanKind::Producer => "producer",
        SpanKind::Consumer => "consumer",
        SpanKind::Internal => "internal",
    }
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

fn events_to_logs(events: SpanEvents) -> Option<Vec<jaeger::Log>> {
    if events.is_empty() {
        None
    } else {
        Some(events.into_iter().map(Into::into).collect())
    }
}

/// Wrap type for errors from opentelemetry jaeger
#[derive(Debug)]
pub enum Error {
    /// Error from thrift agents.
    ///
    /// If the spans was sent to jaeger agent. Refer [AgentPipeline](config::agent::AgentPipeline) for more details.
    /// If the spans was sent to jaeger collector. Refer [CollectorPipeline](config::collector::CollectorPipeline) for more details.
    ThriftAgentError(::thrift::Error),

    /// Pipeline fails because one of the configurations is invalid.
    ConfigError {
        /// the name of the pipeline. It can be `agent`, `collector` or `wasm collector`
        pipeline_name: &'static str,
        /// config name that has the error.
        config_name: &'static str,
        /// the underlying error message.
        reason: String,
    },
}

impl std::error::Error for Error {}

impl From<::thrift::Error> for Error {
    fn from(value: ::thrift::Error) -> Self {
        Error::ThriftAgentError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ThriftAgentError(err) => match err {
                ::thrift::Error::Transport(transport_error) => {
                    write!(
                        f,
                        "thrift agent failed on transportation layer, {}, {}",
                        transport_error, transport_error.message
                    )
                }
                ::thrift::Error::Protocol(protocol_error) => {
                    write!(
                        f,
                        "thrift agent failed on protocol layer, {}, {}",
                        protocol_error, protocol_error.message
                    )
                }
                ::thrift::Error::Application(application_error) => {
                    write!(
                        f,
                        "thrift agent failed on application layer, {}, {}",
                        application_error, application_error.message
                    )
                }
                ::thrift::Error::User(error) => {
                    write!(f, "thrift agent failed, {}", error)
                }
            },
            Error::ConfigError {
                pipeline_name,
                config_name,
                reason,
            } => write!(
                f,
                "{} pipeline fails because one of the configuration {} is invalid. {}",
                pipeline_name, config_name, reason
            ),
        }
    }
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "jaeger"
    }
}

/// Sample the first address provided to designate which IP family to bind the socket to.
/// IP families returned be INADDR_ANY as [`Ipv4Addr::UNSPECIFIED`] or
/// IN6ADDR_ANY as [`Ipv6Addr::UNSPECIFIED`].
fn address_family(addrs: &[SocketAddr]) -> SocketAddr {
    match addrs.first() {
        Some(SocketAddr::V4(_)) | None => SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
        Some(SocketAddr::V6(_)) => SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)),
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::{
        trace::{SpanKind, Status},
        KeyValue,
    };

    use crate::exporter::thrift::jaeger::Tag;
    use crate::exporter::{build_span_tags, OTEL_STATUS_CODE, OTEL_STATUS_DESCRIPTION};

    use super::SPAN_KIND;

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

    #[rustfmt::skip]
    fn get_error_tag_test_data() -> Vec<(Status, Option<&'static str>, Option<&'static str>)>
    {
        // Status, OTEL_STATUS_CODE tag value, OTEL_STATUS_DESCRIPTION tag value
        vec![
            (Status::error(""), Some("ERROR"), None),
            (Status::Unset, None, None),
            // When status is ok, no description should be in span data. This should be ensured by Otel API
            (Status::Ok, Some("OK"), None),
            (Status::error("have message"), Some("ERROR"), Some("have message")),
            (Status::Unset, None, None),
        ]
    }

    #[test]
    fn test_set_status() {
        for (status, status_tag_val, msg_tag_val) in get_error_tag_test_data() {
            let tags = build_span_tags(Vec::new(), None, status, SpanKind::Client);
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
        let mut attributes = Vec::new();
        let user_error = true;
        let user_kind = "server";
        let user_status_description = "Something bad happened";
        let user_status = Status::Error {
            description: user_status_description.into(),
        };
        attributes.push(KeyValue::new("error", user_error));
        attributes.push(KeyValue::new(SPAN_KIND, user_kind));
        attributes.push(KeyValue::new(OTEL_STATUS_CODE, "ERROR"));
        attributes.push(KeyValue::new(
            OTEL_STATUS_DESCRIPTION,
            user_status_description,
        ));
        let tags = build_span_tags(attributes, None, user_status, SpanKind::Client);

        assert!(tags
            .iter()
            .filter(|tag| tag.key.as_str() == "error")
            .all(|tag| tag.v_bool.unwrap()));
        assert_tag_contains(tags.clone(), SPAN_KIND, user_kind);
        assert_tag_contains(tags.clone(), OTEL_STATUS_CODE, "ERROR");
        assert_tag_contains(tags, OTEL_STATUS_DESCRIPTION, user_status_description);
    }

    #[test]
    fn error_message_should_contain_details() {
        let size_limit_err =
            crate::Error::from(::thrift::Error::Protocol(thrift::ProtocolError::new(
                thrift::ProtocolErrorKind::SizeLimit,
                "the error message should contain details".to_string(),
            )));
        assert_eq!(
            format!("{}", size_limit_err),
            "thrift agent failed on protocol layer, message too long, the error message should contain details"
        );
    }
}
