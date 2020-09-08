use serde::Serialize;

#[derive(TypedBuilder, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::model::annotation::Annotation;

    #[test]
    fn test_empty() {
        test_json_serialization(Annotation::builder().build(), "{}");
    }

    #[test]
    fn test_full_annotation() {
        test_json_serialization(
            Annotation::builder()
                .timestamp(1_502_787_600_000_000)
                .value("open-telemetry".to_owned())
                .build(),
            "{\"timestamp\":1502787600000000,\"value\":\"open-telemetry\"}",
        );
    }

    fn test_json_serialization(annotation: Annotation, desired: &str) {
        let result = serde_json::to_string(&annotation).unwrap();
        assert_eq!(result, desired.to_owned());
    }
}
