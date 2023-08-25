use opentelemetry::trace::Event;
use serde::Serialize;
use std::time::{Duration, SystemTime};

#[derive(TypedBuilder, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Annotation {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

/// Converts `Event` into an `annotation::Annotation`
impl From<Event> for Annotation {
    fn from(event: Event) -> Annotation {
        let timestamp = event
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_micros() as u64;

        Annotation::builder()
            .timestamp(timestamp)
            .value(event.name.into())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::model::annotation::Annotation;

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
