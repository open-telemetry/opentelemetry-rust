// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
pub mod tonic {
    use crate::proto::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            metric::Data, number_data_point, AggregationTemporality, Gauge, Histogram,
            HistogramDataPoint, InstrumentationLibraryMetrics, Metric, NumberDataPoint,
            ResourceMetrics, Sum,
        },
    };
    use opentelemetry::metrics::{MetricsError, Number, NumberKind};
    use opentelemetry::sdk::export::metrics::{
        Count, ExportKind, ExportKindFor, Histogram as SdkHistogram, LastValue, Max, Min, Points,
        Record, Sum as SdkSum,
    };
    use opentelemetry::sdk::metrics::aggregators::{
        ArrayAggregator, HistogramAggregator, LastValueAggregator, MinMaxSumCountAggregator,
        SumAggregator,
    };

    use crate::transform::common::to_nanos;
    use opentelemetry::sdk::InstrumentationLibrary;
    use opentelemetry::{Key, Value};
    use std::collections::{BTreeMap, HashMap};


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