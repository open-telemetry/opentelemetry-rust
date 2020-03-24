use serde::Serialize;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(TypedBuilder, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Endpoint {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    service_name: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv4: Option<Ipv4Addr>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv6: Option<Ipv6Addr>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

#[cfg(test)]
mod endpoint_serialization_tests {
    use crate::model::endpoint::Endpoint;
    use std::net::Ipv4Addr;

    #[test]
    fn test_empty() {
        test_json_serialization(Endpoint::builder().build(), "{}");
    }

    #[test]
    fn test_ipv4_empty() {
        test_json_serialization(
            Endpoint::builder()
                .service_name("open-telemetry".to_owned())
                .ipv4(Ipv4Addr::new(127, 0, 0, 1))
                .port(8080)
                .build(),
            "{\"serviceName\":\"open-telemetry\",\"ipv4\":\"127.0.0.1\",\"port\":8080}",
        );
    }

    fn test_json_serialization(endpoint: Endpoint, desired: &str) {
        let result = serde_json::to_string(&endpoint).unwrap();
        assert_eq!(result, desired.to_owned());
    }
}
