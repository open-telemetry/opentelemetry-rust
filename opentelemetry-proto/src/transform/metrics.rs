// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use crate::proto::tonic::{common::v1::KeyValue, metrics::v1::AggregationTemporality};
    use crate::tonic::metrics::v1::{exemplar, number_data_point};
    use opentelemetry_api::metrics::MetricsError;
    use opentelemetry_api::{Key, Value};
    use opentelemetry_sdk::metrics::data::Temporality;

    impl From<u64> for exemplar::Value {
        fn from(value: u64) -> Self {
            exemplar::Value::AsInt(i64::try_from(value).unwrap_or_default())
        }
    }

    impl From<i64> for exemplar::Value {
        fn from(value: i64) -> Self {
            exemplar::Value::AsInt(value)
        }
    }

    impl From<f64> for exemplar::Value {
        fn from(value: f64) -> Self {
            exemplar::Value::AsDouble(value)
        }
    }

    impl From<u64> for number_data_point::Value {
        fn from(value: u64) -> Self {
            number_data_point::Value::AsInt(i64::try_from(value).unwrap_or_default())
        }
    }

    impl From<i64> for number_data_point::Value {
        fn from(value: i64) -> Self {
            number_data_point::Value::AsInt(value)
        }
    }

    impl From<f64> for number_data_point::Value {
        fn from(value: f64) -> Self {
            number_data_point::Value::AsDouble(value)
        }
    }

    impl From<(&Key, &Value)> for KeyValue {
        fn from(kv: (&Key, &Value)) -> Self {
            KeyValue {
                key: kv.0.to_string(),
                value: Some(kv.1.clone().into()),
            }
        }
    }

    impl From<&opentelemetry_api::KeyValue> for KeyValue {
        fn from(kv: &opentelemetry_api::KeyValue) -> Self {
            KeyValue {
                key: kv.key.to_string(),
                value: Some(kv.value.clone().into()),
            }
        }
    }

    impl From<Temporality> for AggregationTemporality {
        fn from(temporality: Temporality) -> Self {
            match temporality {
                Temporality::Cumulative => AggregationTemporality::Cumulative,
                Temporality::Delta => AggregationTemporality::Delta,
                other => {
                    opentelemetry_api::global::handle_error(MetricsError::Other(format!(
                        "Unknown temporality {:?}, using default instead.",
                        other
                    )));
                    AggregationTemporality::Cumulative
                }
            }
        }
    }
}
