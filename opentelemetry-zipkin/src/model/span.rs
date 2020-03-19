use crate::model::{annotation::Annotation, endpoint::Endpoint};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ListOfSpans(pub(crate) Vec<Span>);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Kind {
    Client,
    Server,
    Producer,
    Consumer,
}

#[derive(TypedBuilder, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
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
mod span_serialization_tests {
    use crate::model::annotation::Annotation;
    use crate::model::endpoint::Endpoint;
    use crate::model::span::{Kind, Span};
    use std::collections::HashMap;
    use std::net::Ipv4Addr;

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
}
