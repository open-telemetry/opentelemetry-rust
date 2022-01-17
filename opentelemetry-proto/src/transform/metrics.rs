// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
#[cfg(feature = "gen-tonic")]
pub mod tonic {
    use crate::proto::tonic::{
        common::v1::KeyValue,
        metrics::v1::{number_data_point, AggregationTemporality},
    };
    use opentelemetry::metrics::{Number, NumberKind};
    use opentelemetry::sdk::export::metrics::ExportKind;

    use opentelemetry::{Key, Value};

    /// Convert Number to target type based on it's NumberKind.
    pub trait FromNumber {
        fn from_number(number: Number, number_kind: &NumberKind) -> Self;
    }

    impl FromNumber for number_data_point::Value {
        fn from_number(number: Number, number_kind: &NumberKind) -> Self {
            match &number_kind {
                NumberKind::I64 | NumberKind::U64 => {
                    number_data_point::Value::AsInt(number.to_i64(number_kind))
                }
                NumberKind::F64 => number_data_point::Value::AsDouble(number.to_f64(number_kind)),
            }
        }
    }

    impl From<(&Key, &Value)> for KeyValue {
        fn from(kv: (&Key, &Value)) -> Self {
            KeyValue {
                key: kv.0.clone().into(),
                value: Some(kv.1.clone().into()),
            }
        }
    }

    impl From<ExportKind> for AggregationTemporality {
        fn from(kind: ExportKind) -> Self {
            match kind {
                ExportKind::Cumulative => AggregationTemporality::Cumulative,
                ExportKind::Delta => AggregationTemporality::Delta,
            }
        }
    }
}
