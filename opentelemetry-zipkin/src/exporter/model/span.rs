use crate::exporter::model::{annotation::Annotation, endpoint::Endpoint};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Kind {
    Client,
    Server,
    Producer,
    Consumer,
}

#[derive(TypedBuilder, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Span {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<Kind>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<u64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    local_endpoint: Option<Endpoint>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_endpoint: Option<Endpoint>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<Vec<Annotation>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<HashMap<String, String>>,
    #[builder(default = false)]
    debug: bool,
    #[builder(default = false)]
    shared: bool,
}

#[cfg(test)]
mod tests {
    use crate::exporter::model::annotation::Annotation;
    use crate::exporter::model::endpoint::Endpoint;
    use crate::exporter::model::span::{Kind, Span};
    use crate::exporter::model::{into_zipkin_span, OTEL_ERROR_DESCRIPTION, OTEL_STATUS_CODE};
    use opentelemetry::sdk::export::trace::SpanData;
    use opentelemetry::sdk::trace::{EvictedHashMap, EvictedQueue};
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, StatusCode, TraceId};
    use std::collections::HashMap;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use std::time::SystemTime;

    #[test]
    fn test_empty() {
        test_json_serialization(
            Span::builder().build(),
            "{\"debug\":false,\"shared\":false}",
        );
    }

    #[test]
    fn test_full_span() {
        let mut tags = HashMap::new();
        tags.insert("a".to_owned(), "b".to_owned());
        test_json_serialization(
            Span::builder()
                .trace_id("4e441824ec2b6a44ffdc9bb9a6453df3".to_owned())
                .parent_id("ffdc9bb9a6453df3".to_owned())
                .id("efdc9cd9a1849df3".to_owned())
                .kind(Some(Kind::Server))
                .name("main".to_owned())
                .timestamp(1_502_787_600_000_000)
                .duration(150_000)
                .local_endpoint(
                    Endpoint::builder()
                        .service_name("remote-service".to_owned())
                        .ipv4(Ipv4Addr::new(192, 168, 0, 1))
                        .port(8080)
                        .build()
                )
                .remote_endpoint(
                    Endpoint::builder()
                        .service_name("open-telemetry".to_owned())
                        .ipv4(Ipv4Addr::new(127, 0, 0, 1))
                        .port(8080)
                        .build()
                )
                .annotations(vec![
                    Annotation::builder()
                        .timestamp(1_502_780_000_000_000)
                        .value("interesting event".to_string())
                        .build()
                ])
                .tags(tags)
                .build(),
            "{\"traceId\":\"4e441824ec2b6a44ffdc9bb9a6453df3\",\"parentId\":\"ffdc9bb9a6453df3\",\"id\":\"efdc9cd9a1849df3\",\"kind\":\"SERVER\",\"name\":\"main\",\"timestamp\":1502787600000000,\"duration\":150000,\"localEndpoint\":{\"serviceName\":\"remote-service\",\"ipv4\":\"192.168.0.1\",\"port\":8080},\"remoteEndpoint\":{\"serviceName\":\"open-telemetry\",\"ipv4\":\"127.0.0.1\",\"port\":8080},\"annotations\":[{\"timestamp\":1502780000000000,\"value\":\"interesting event\"}],\"tags\":{\"a\":\"b\"},\"debug\":false,\"shared\":false}",
        );
    }

    fn test_json_serialization(span: Span, desired: &str) {
        let result = serde_json::to_string(&span).unwrap();
        assert_eq!(result, desired.to_owned());
    }

    fn assert_tag_contains(
        tags: &HashMap<String, String>,
        key: &'static str,
        expected_val: Option<&'static str>,
    ) {
        let val = tags.get::<String>(&key.to_string()).map(|s| s.as_str());
        assert_eq!(
            val,
            expected_val,
            "expect value of key {} to be {}, but got {}",
            key,
            expected_val.unwrap_or("none"),
            val.unwrap_or("none")
        );
    }

    fn get_set_status_test_data() -> Vec<(
        StatusCode,
        String,
        Option<&'static str>,
        Option<&'static str>,
    )> {
        // status code, status message, whether OTEL_STATUS_CODE is set, whether OTEL_ERROR_DESCRIPTION is set, whether error tag is set
        vec![
            (StatusCode::Ok, "".into(), Some("OK"), None),
            (StatusCode::Error, "".into(), Some("ERROR"), Some("")),
            (
                StatusCode::Error,
                "error msg".into(),
                Some("ERROR"),
                Some("error msg"),
            ),
            (StatusCode::Unset, "whatever".into(), None, None),
        ]
    }

    #[test]
    fn test_set_status() -> Result<(), Box<dyn std::error::Error>> {
        for (status_code, status_msg, status_tag_val, status_msg_tag_val) in
            get_set_status_test_data()
        {
            let span_data = SpanData {
                span_context: SpanContext::new(
                    TraceId::from_u128(1),
                    SpanId::from_u64(1),
                    0,
                    false,
                    Default::default(),
                ),
                parent_span_id: SpanId::from_u64(1),
                span_kind: SpanKind::Client,
                name: "".to_string(),
                start_time: SystemTime::now(),
                end_time: SystemTime::now(),
                attributes: EvictedHashMap::new(20, 20),
                message_events: EvictedQueue::new(20),
                links: EvictedQueue::new(20),
                status_code,
                status_message: status_msg,
                resource: Arc::new(Default::default()),
                instrumentation_lib: Default::default(),
            };
            let local_endpoint = Endpoint::new("test".into(), None);
            let span = into_zipkin_span(local_endpoint, span_data);
            if let Some(tags) = span.tags.as_ref() {
                assert_tag_contains(tags, OTEL_STATUS_CODE, status_tag_val);
                assert_tag_contains(tags, OTEL_ERROR_DESCRIPTION, status_msg_tag_val);
            };
        }

        Ok(())
    }
}
