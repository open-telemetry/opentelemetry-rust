// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use std::any::Any;
    use std::fmt;

    use opentelemetry_api::{global, metrics::MetricsError, Key, Value};
    use opentelemetry_sdk::metrics::data::{
        self, Exemplar as SdkExemplar, Gauge as SdkGauge, Histogram as SdkHistogram,
        Metric as SdkMetric, ScopeMetrics as SdkScopeMetrics, Sum as SdkSum, Temporality,
    };
    use opentelemetry_sdk::Resource as SdkResource;

    use crate::proto::tonic::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            exemplar, exemplar::Value as TonicExemplarValue, metric::Data as TonicMetricData,
            number_data_point, number_data_point::Value as TonicDataPointValue,
            AggregationTemporality as TonicTemporality, AggregationTemporality,
            DataPointFlags as TonicDataPointFlags, Exemplar as TonicExemplar, Gauge as TonicGauge,
            Histogram as TonicHistogram, HistogramDataPoint as TonicHistogramDataPoint,
            Metric as TonicMetric, NumberDataPoint as TonicNumberDataPoint,
            ResourceMetrics as TonicResourceMetrics, ScopeMetrics as TonicScopeMetrics,
            Sum as TonicSum,
        },
        resource::v1::Resource as TonicResource,
    };
    use crate::transform::common::to_nanos;

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

    impl From<&data::ResourceMetrics> for ExportMetricsServiceRequest {
        fn from(rm: &data::ResourceMetrics) -> Self {
            ExportMetricsServiceRequest {
                resource_metrics: vec![TonicResourceMetrics {
                    resource: Some((&rm.resource).into()),
                    scope_metrics: rm.scope_metrics.iter().map(Into::into).collect(),
                    schema_url: rm.resource.schema_url().map(Into::into).unwrap_or_default(),
                }],
            }
        }
    }

    impl From<&SdkResource> for TonicResource {
        fn from(resource: &SdkResource) -> Self {
            TonicResource {
                attributes: resource.iter().map(Into::into).collect(),
                dropped_attributes_count: 0,
            }
        }
    }

    impl From<&SdkScopeMetrics> for TonicScopeMetrics {
        fn from(sm: &SdkScopeMetrics) -> Self {
            TonicScopeMetrics {
                scope: Some((&sm.scope).into()),
                metrics: sm.metrics.iter().map(Into::into).collect(),
                schema_url: sm
                    .scope
                    .schema_url
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            }
        }
    }

    impl From<&SdkMetric> for TonicMetric {
        fn from(metric: &SdkMetric) -> Self {
            TonicMetric {
                name: metric.name.to_string(),
                description: metric.description.to_string(),
                unit: metric.unit.as_str().to_string(),
                data: metric.data.as_any().try_into().ok(),
            }
        }
    }

    impl TryFrom<&dyn Any> for TonicMetricData {
        type Error = ();

        fn try_from(data: &dyn Any) -> Result<Self, Self::Error> {
            if let Some(hist) = data.downcast_ref::<SdkHistogram<i64>>() {
                Ok(TonicMetricData::Histogram(hist.into()))
            } else if let Some(hist) = data.downcast_ref::<SdkHistogram<u64>>() {
                Ok(TonicMetricData::Histogram(hist.into()))
            } else if let Some(hist) = data.downcast_ref::<SdkHistogram<f64>>() {
                Ok(TonicMetricData::Histogram(hist.into()))
            } else if let Some(sum) = data.downcast_ref::<SdkSum<u64>>() {
                Ok(TonicMetricData::Sum(sum.into()))
            } else if let Some(sum) = data.downcast_ref::<SdkSum<i64>>() {
                Ok(TonicMetricData::Sum(sum.into()))
            } else if let Some(sum) = data.downcast_ref::<SdkSum<f64>>() {
                Ok(TonicMetricData::Sum(sum.into()))
            } else if let Some(gauge) = data.downcast_ref::<SdkGauge<u64>>() {
                Ok(TonicMetricData::Gauge(gauge.into()))
            } else if let Some(gauge) = data.downcast_ref::<SdkGauge<i64>>() {
                Ok(TonicMetricData::Gauge(gauge.into()))
            } else if let Some(gauge) = data.downcast_ref::<SdkGauge<f64>>() {
                Ok(TonicMetricData::Gauge(gauge.into()))
            } else {
                global::handle_error(MetricsError::Other("unknown aggregator".into()));
                Err(())
            }
        }
    }

    trait Numeric: Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy {
        // lossy at large values for u64 and i64 but otlp histograms only handle float values
        fn into_f64(self) -> f64;
    }

    impl Numeric for u64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    impl Numeric for i64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    impl Numeric for f64 {
        fn into_f64(self) -> f64 {
            self
        }
    }

    impl<T> From<&SdkHistogram<T>> for TonicHistogram
    where
        T: Numeric,
    {
        fn from(hist: &SdkHistogram<T>) -> Self {
            TonicHistogram {
                data_points: hist
                    .data_points
                    .iter()
                    .map(|dp| TonicHistogramDataPoint {
                        attributes: dp.attributes.iter().map(Into::into).collect(),
                        start_time_unix_nano: to_nanos(dp.start_time),
                        time_unix_nano: to_nanos(dp.time),
                        count: dp.count,
                        sum: Some(dp.sum.into_f64()),
                        bucket_counts: dp.bucket_counts.clone(),
                        explicit_bounds: dp.bounds.clone(),
                        exemplars: dp.exemplars.iter().map(Into::into).collect(),
                        flags: TonicDataPointFlags::default() as u32,
                        min: dp.min.map(Numeric::into_f64),
                        max: dp.max.map(Numeric::into_f64),
                    })
                    .collect(),
                aggregation_temporality: TonicTemporality::from(hist.temporality).into(),
            }
        }
    }

    impl<T> From<&SdkSum<T>> for TonicSum
    where
        T: fmt::Debug + Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy,
    {
        fn from(sum: &SdkSum<T>) -> Self {
            TonicSum {
                data_points: sum
                    .data_points
                    .iter()
                    .map(|dp| TonicNumberDataPoint {
                        attributes: dp.attributes.iter().map(Into::into).collect(),
                        start_time_unix_nano: dp.start_time.map(to_nanos).unwrap_or_default(),
                        time_unix_nano: dp.time.map(to_nanos).unwrap_or_default(),
                        exemplars: dp.exemplars.iter().map(Into::into).collect(),
                        flags: TonicDataPointFlags::default() as u32,
                        value: Some(dp.value.into()),
                    })
                    .collect(),
                aggregation_temporality: TonicTemporality::from(sum.temporality).into(),
                is_monotonic: sum.is_monotonic,
            }
        }
    }

    impl<T> From<&SdkGauge<T>> for TonicGauge
    where
        T: fmt::Debug + Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy,
    {
        fn from(gauge: &SdkGauge<T>) -> Self {
            TonicGauge {
                data_points: gauge
                    .data_points
                    .iter()
                    .map(|dp| TonicNumberDataPoint {
                        attributes: dp.attributes.iter().map(Into::into).collect(),
                        start_time_unix_nano: dp.start_time.map(to_nanos).unwrap_or_default(),
                        time_unix_nano: dp.time.map(to_nanos).unwrap_or_default(),
                        exemplars: dp.exemplars.iter().map(Into::into).collect(),
                        flags: TonicDataPointFlags::default() as u32,
                        value: Some(dp.value.into()),
                    })
                    .collect(),
            }
        }
    }

    impl<T> From<&SdkExemplar<T>> for TonicExemplar
    where
        T: Into<TonicExemplarValue> + Copy,
    {
        fn from(ex: &SdkExemplar<T>) -> Self {
            TonicExemplar {
                filtered_attributes: ex
                    .filtered_attributes
                    .iter()
                    .map(|kv| (&kv.key, &kv.value).into())
                    .collect(),
                time_unix_nano: to_nanos(ex.time),
                span_id: ex.span_id.into(),
                trace_id: ex.trace_id.into(),
                value: Some(ex.value.into()),
            }
        }
    }
}
