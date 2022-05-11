use crate::exporter::ModelConfig;
use http::uri;
use opentelemetry::sdk::export::{
    trace::{self, SpanData},
    ExportError,
};
use std::fmt::Debug;
use url::ParseError;

mod v03;
mod v05;

// https://github.com/DataDog/dd-trace-js/blob/c89a35f7d27beb4a60165409376e170eacb194c5/packages/dd-trace/src/constants.js#L4
static SAMPLING_PRIORITY_KEY: &str = "_sampling_priority_v1";

/// Custom mapping between opentelemetry spans and datadog spans.
///
/// User can provide custom function to change the mapping. It currently supports customizing the following
/// fields in Datadog span protocol.
///
/// |field name|default value|
/// |---------------|-------------|
/// |service name| service name configuration from [`ModelConfig`]|
/// |name | opentelemetry instrumentation library name |
/// |resource| opentelemetry name|
///
/// The function takes a reference to [`SpanData`]() and a reference to [`ModelConfig`]() as parameters.
/// It should return a `&str` which will be used as the value for the field.
///
/// If no custom mapping is provided. Default mapping detailed above will be used.
///
/// For example,
/// ```no_run
/// use opentelemetry_datadog::{ApiVersion, new_pipeline};
/// fn main() -> Result<(), opentelemetry::trace::TraceError> {
///    let tracer = new_pipeline()
///            .with_service_name("my_app")
///            .with_version(ApiVersion::Version05)
///            // the custom mapping below will change the all spans' name to datadog spans
///            .with_name_mapping(|span, model_config|{
///                 "datadog spans"
///             })
///            .with_agent_endpoint("http://localhost:8126")
///            .install_batch(opentelemetry::runtime::Tokio)?;
///
///    Ok(())
/// }
/// ```
pub type FieldMappingFn = dyn for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + Send + Sync;

pub(crate) type FieldMapping = std::sync::Arc<FieldMappingFn>;

// Datadog uses some magic tags in their models. There is no recommended mapping defined in
// opentelemetry spec. Below is default mapping we gonna uses. Users can override it by providing
// their own implementations
fn default_service_name_mapping<'a>(_span: &'a SpanData, config: &'a ModelConfig) -> &'a str {
    config.service_name.as_str()
}

fn default_name_mapping<'a>(span: &'a SpanData, _config: &'a ModelConfig) -> &'a str {
    span.instrumentation_lib.name.as_ref()
}

fn default_resource_mapping<'a>(span: &'a SpanData, _config: &'a ModelConfig) -> &'a str {
    span.name.as_ref()
}

/// Wrap type for errors from opentelemetry datadog exporter
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Message pack error
    #[error("message pack error")]
    MessagePackError,
    /// No http client founded. User should provide one or enable features
    #[error("http client must be set, users can enable reqwest or surf feature to use http client implementation within create")]
    NoHttpClient,
    /// Http requests failed with following errors
    #[error(transparent)]
    RequestError(#[from] http::Error),
    /// The Uri was invalid
    #[error("invalid url {0}")]
    InvalidUri(String),
    /// Other errors
    #[error("{0}")]
    Other(String),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "datadog"
    }
}

impl From<rmp::encode::ValueWriteError> for Error {
    fn from(_: rmp::encode::ValueWriteError) -> Self {
        Self::MessagePackError
    }
}

impl From<url::ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::InvalidUri(err.to_string())
    }
}

impl From<uri::InvalidUri> for Error {
    fn from(err: uri::InvalidUri) -> Self {
        Self::InvalidUri(err.to_string())
    }
}

/// Version of datadog trace ingestion API
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum ApiVersion {
    /// Version 0.3
    Version03,
    /// Version 0.5 - requires datadog-agent v7.22.0 or above
    Version05,
}

impl ApiVersion {
    pub(crate) fn path(self) -> &'static str {
        match self {
            ApiVersion::Version03 => "/v0.3/traces",
            ApiVersion::Version05 => "/v0.5/traces",
        }
    }

    pub(crate) fn content_type(self) -> &'static str {
        match self {
            ApiVersion::Version03 => "application/msgpack",
            ApiVersion::Version05 => "application/msgpack",
        }
    }

    pub(crate) fn encode(
        self,
        model_config: &ModelConfig,
        traces: Vec<Vec<trace::SpanData>>,
        get_service_name: Option<FieldMapping>,
        get_name: Option<FieldMapping>,
        get_resource: Option<FieldMapping>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::Version03 => v03::encode(
                model_config,
                traces,
                |span, config| match &get_service_name {
                    Some(f) => f(span, config),
                    None => default_service_name_mapping(span, config),
                },
                |span, config| match &get_name {
                    Some(f) => f(span, config),
                    None => default_name_mapping(span, config),
                },
                |span, config| match &get_resource {
                    Some(f) => f(span, config),
                    None => default_resource_mapping(span, config),
                },
            ),
            Self::Version05 => v05::encode(
                model_config,
                traces,
                |span, config| match &get_service_name {
                    Some(f) => f(span, config),
                    None => default_service_name_mapping(span, config),
                },
                |span, config| match &get_name {
                    Some(f) => f(span, config),
                    None => default_name_mapping(span, config),
                },
                |span, config| match &get_resource {
                    Some(f) => f(span, config),
                    None => default_resource_mapping(span, config),
                },
            ),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use opentelemetry::sdk::InstrumentationLibrary;
    use opentelemetry::sdk::{self, Resource};
    use opentelemetry::{
        trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState},
        Key,
    };
    use std::borrow::Cow;
    use std::time::{Duration, SystemTime};

    fn get_traces() -> Vec<Vec<trace::SpanData>> {
        vec![vec![get_span(7, 1, 99)]]
    }

    pub(crate) fn get_span(trace_id: u128, parent_span_id: u64, span_id: u64) -> trace::SpanData {
        let span_context = SpanContext::new(
            TraceId::from_u128(trace_id),
            SpanId::from_u64(span_id),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        let start_time = SystemTime::UNIX_EPOCH;
        let end_time = start_time.checked_add(Duration::from_secs(1)).unwrap();

        let capacity = 3;
        let mut attributes = sdk::trace::EvictedHashMap::new(capacity, capacity as usize);
        attributes.insert(Key::new("span.type").string("web"));

        let events = sdk::trace::EvictedQueue::new(capacity);
        let links = sdk::trace::EvictedQueue::new(capacity);

        trace::SpanData {
            span_context,
            parent_span_id: SpanId::from_u64(parent_span_id),
            span_kind: SpanKind::Client,
            name: "resource".into(),
            start_time,
            end_time,
            attributes,
            events,
            links,
            status: Status::Ok,
            resource: Cow::Owned(Resource::empty()),
            instrumentation_lib: InstrumentationLibrary::new("component", None, None),
        }
    }

    #[test]
    fn test_encode_v03() -> Result<(), Box<dyn std::error::Error>> {
        let traces = get_traces();
        let model_config = ModelConfig {
            service_name: "service_name".to_string(),
            ..Default::default()
        };
        let encoded = base64::encode(ApiVersion::Version03.encode(
            &model_config,
            traces,
            None,
            None,
            None,
        )?);

        assert_eq!(encoded.as_str(), "kZGLpHR5cGWjd2Vip3NlcnZpY2Wsc2VydmljZV9uYW1lpG5hbWWpY29tcG9uZW50qHJlc291cmNlqHJlc291cmNlqHRyYWNlX2lkzwAAAAAAAAAHp3NwYW5faWTPAAAAAAAAAGOpcGFyZW50X2lkzwAAAAAAAAABpXN0YXJ00wAAAAAAAAAAqGR1cmF0aW9u0wAAAAA7msoApWVycm9y0gAAAACkbWV0YYGpc3Bhbi50eXBlo3dlYqdtZXRyaWNzgbVfc2FtcGxpbmdfcHJpb3JpdHlfdjHLAAAAAAAAAAA=");

        Ok(())
    }

    #[test]
    fn test_encode_v05() -> Result<(), Box<dyn std::error::Error>> {
        let traces = get_traces();
        let model_config = ModelConfig {
            service_name: "service_name".to_string(),
            ..Default::default()
        };
        let encoded = base64::encode(ApiVersion::Version05.encode(
            &model_config,
            traces,
            None,
            None,
            None,
        )?);

        assert_eq!(encoded.as_str(),
                   "kpajd2VirHNlcnZpY2VfbmFtZaljb21wb25lbnSocmVzb3VyY2Wpc3Bhbi50eXBltV9zYW1wbGluZ19wcmlvcml0eV92MZGRnM4AAAABzgAAAALOAAAAA88AAAAAAAAAB88AAAAAAAAAY88AAAAAAAAAAdMAAAAAAAAAANMAAAAAO5rKANIAAAAAgc4AAAAEzgAAAACBzgAAAAXLAAAAAAAAAADOAAAAAA==");

        Ok(())
    }
}
