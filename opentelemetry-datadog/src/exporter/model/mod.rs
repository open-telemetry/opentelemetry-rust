use opentelemetry::sdk::export::{trace, ExportError};

mod v03;
mod v05;

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
    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
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

/// Version of datadog trace ingestion API
#[derive(Debug, Copy, Clone)]
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
        service_name: &str,
        traces: Vec<Vec<trace::SpanData>>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::Version03 => v03::encode(service_name, traces),
            Self::Version05 => v05::encode(service_name, traces),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use opentelemetry::sdk;
    use opentelemetry::sdk::InstrumentationLibrary;
    use opentelemetry::{
        trace::{SpanContext, SpanId, SpanKind, StatusCode, TraceFlags, TraceId, TraceState},
        Key,
    };
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
            status_code: StatusCode::Ok,
            status_message: "".into(),
            resource: None,
            instrumentation_lib: InstrumentationLibrary::new("component", None, None),
        }
    }

    #[test]
    fn test_encode_v03() -> Result<(), Box<dyn std::error::Error>> {
        let traces = get_traces();
        let encoded = base64::encode(ApiVersion::Version03.encode("service_name", traces)?);

        assert_eq!(encoded.as_str(), "kZGLpHR5cGWjd2Vip3NlcnZpY2Wsc2VydmljZV9uYW1lpG5hbWWpY29tcG9uZW50qHJlc291cmNlqHJlc291cmNlqHRyYWNlX2lkzwAAAAAAAAAHp3NwYW5faWTPAAAAAAAAAGOpcGFyZW50X2lkzwAAAAAAAAABpXN0YXJ00wAAAAAAAAAAqGR1cmF0aW9u0wAAAAA7msoApWVycm9y0gAAAACkbWV0YYGpc3Bhbi50eXBlo3dlYg==");

        Ok(())
    }

    #[test]
    fn test_encode_v05() -> Result<(), Box<dyn std::error::Error>> {
        let traces = get_traces();
        let encoded = base64::encode(ApiVersion::Version05.encode("service_name", traces)?);

        assert_eq!(encoded.as_str(), "kpWsc2VydmljZV9uYW1lo3dlYqljb21wb25lbnSocmVzb3VyY2Wpc3Bhbi50eXBlkZGczgAAAADOAAAAAs4AAAADzwAAAAAAAAAHzwAAAAAAAABjzwAAAAAAAAAB0wAAAAAAAAAA0wAAAAA7msoA0gAAAACBzgAAAATOAAAAAYDOAAAAAQ==");

        Ok(())
    }
}
