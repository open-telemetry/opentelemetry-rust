// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
pub mod tonic {
    use std::fmt::Debug;

    use opentelemetry::{otel_debug, Key, Value};
    use opentelemetry_sdk::metrics::data::{
        AggregatedMetrics, Exemplar as SdkExemplar,
        ExponentialHistogram as SdkExponentialHistogram, Gauge as SdkGauge,
        Histogram as SdkHistogram, Metric as SdkMetric, MetricData, ResourceMetrics,
        ScopeMetrics as SdkScopeMetrics, Sum as SdkSum,
    };
    use opentelemetry_sdk::metrics::Temporality;
    use opentelemetry_sdk::Resource as SdkResource;

    use opentelemetry_proto::tonic::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            exemplar, exemplar::Value as TonicExemplarValue,
            exponential_histogram_data_point::Buckets as TonicBuckets,
            metric::Data as TonicMetricData, number_data_point,
            number_data_point::Value as TonicDataPointValue,
            AggregationTemporality,
            DataPointFlags as TonicDataPointFlags, Exemplar as TonicExemplar,
            ExponentialHistogram as TonicExponentialHistogram,
            ExponentialHistogramDataPoint as TonicExponentialHistogramDataPoint,
            Gauge as TonicGauge, Histogram as TonicHistogram,
            HistogramDataPoint as TonicHistogramDataPoint, Metric as TonicMetric,
            NumberDataPoint as TonicNumberDataPoint, ResourceMetrics as TonicResourceMetrics,
            ScopeMetrics as TonicScopeMetrics, Sum as TonicSum,
        },
        resource::v1::Resource as TonicResource,
    };
    use crate::transform::common::{
        to_nanos,
        tonic::{instrumentation_scope_from_scope_ref_and_target, value_to_any_value},
    };

    pub fn exemplar_value_from_u64(value: u64) -> exemplar::Value {
        exemplar::Value::AsInt(i64::try_from(value).unwrap_or_default())
    }

    pub fn exemplar_value_from_i64(value: i64) -> exemplar::Value {
        exemplar::Value::AsInt(value)
    }

    pub fn exemplar_value_from_f64(value: f64) -> exemplar::Value {
        exemplar::Value::AsDouble(value)
    }

    pub fn number_data_point_value_from_u64(value: u64) -> number_data_point::Value {
        number_data_point::Value::AsInt(i64::try_from(value).unwrap_or_default())
    }

    pub fn number_data_point_value_from_i64(value: i64) -> number_data_point::Value {
        number_data_point::Value::AsInt(value)
    }

    pub fn number_data_point_value_from_f64(value: f64) -> number_data_point::Value {
        number_data_point::Value::AsDouble(value)
    }

    pub fn key_value_from_key_value_ref(kv: (&Key, &Value)) -> KeyValue {
        KeyValue {
            key: kv.0.to_string(),
            value: Some(value_to_any_value(kv.1.clone())),
        }
    }

    pub fn key_value_from_otel_key_value(kv: &opentelemetry::KeyValue) -> KeyValue {
        KeyValue {
            key: kv.key.to_string(),
            value: Some(value_to_any_value(kv.value.clone())),
        }
    }

    pub fn temporality_to_aggregation_temporality(temporality: Temporality) -> AggregationTemporality {
        match temporality {
            Temporality::Cumulative => AggregationTemporality::Cumulative,
            Temporality::Delta => AggregationTemporality::Delta,
            other => {
                otel_debug!(
                    name: "AggregationTemporality::Unknown",
                    message = "Unknown temporality,using default instead.",
                    unknown_temporality = format!("{:?}", other),
                    default_temporality = format!("{:?}", Temporality::Cumulative)
                );
                AggregationTemporality::Cumulative
            }
        }
    }

    pub fn resource_metrics_to_export_request(rm: &ResourceMetrics) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![TonicResourceMetrics {
                resource: Some(sdk_resource_to_tonic_resource(rm.resource())),
                scope_metrics: rm.scope_metrics().map(sdk_scope_metrics_to_tonic_scope_metrics).collect(),
                schema_url: rm
                    .resource()
                    .schema_url()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            }],
        }
    }

    pub fn sdk_resource_to_tonic_resource(resource: &SdkResource) -> TonicResource {
        TonicResource {
            attributes: resource.iter().map(key_value_from_key_value_ref).collect(),
            dropped_attributes_count: 0,
            entity_refs: vec![], // internal and currently unused
        }
    }

    pub fn sdk_scope_metrics_to_tonic_scope_metrics(sm: &SdkScopeMetrics) -> TonicScopeMetrics {
        TonicScopeMetrics {
            scope: Some(instrumentation_scope_from_scope_ref_and_target(sm.scope(), None)),
            metrics: sm.metrics().map(sdk_metric_to_tonic_metric).collect(),
            schema_url: sm
                .scope()
                .schema_url()
                .map(ToOwned::to_owned)
                .unwrap_or_default(),
        }
    }

    pub fn sdk_metric_to_tonic_metric(metric: &SdkMetric) -> TonicMetric {
        TonicMetric {
            name: metric.name().to_string(),
            description: metric.description().to_string(),
            unit: metric.unit().to_string(),
            metadata: vec![], // internal and currently unused
            data: Some(match metric.data() {
                AggregatedMetrics::F64(data) => metric_data_to_tonic_metric_data(data),
                AggregatedMetrics::U64(data) => metric_data_to_tonic_metric_data(data),
                AggregatedMetrics::I64(data) => metric_data_to_tonic_metric_data(data),
            }),
        }
    }

    pub fn metric_data_to_tonic_metric_data<T>(data: &MetricData<T>) -> TonicMetricData
    where
        T: Numeric + Debug,
    {
        match data {
            MetricData::Gauge(gauge) => TonicMetricData::Gauge(sdk_gauge_to_tonic_gauge(gauge)),
            MetricData::Sum(sum) => TonicMetricData::Sum(sdk_sum_to_tonic_sum(sum)),
            MetricData::Histogram(hist) => TonicMetricData::Histogram(sdk_histogram_to_tonic_histogram(hist)),
            MetricData::ExponentialHistogram(hist) => {
                TonicMetricData::ExponentialHistogram(sdk_exponential_histogram_to_tonic_exponential_histogram(hist))
            }
        }
    }

    trait Numeric: Copy {
        // lossy at large values for u64 and i64 but otlp histograms only handle float values
        fn into_f64(self) -> f64;
        fn to_data_point_value(self) -> TonicDataPointValue;
        fn to_exemplar_value(self) -> TonicExemplarValue;
    }

    impl Numeric for u64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
        
        fn to_data_point_value(self) -> TonicDataPointValue {
            number_data_point_value_from_u64(self)
        }
        
        fn to_exemplar_value(self) -> TonicExemplarValue {
            exemplar_value_from_u64(self)
        }
    }

    impl Numeric for i64 {
        fn into_f64(self) -> f64 {
            self as f64
        }
        
        fn to_data_point_value(self) -> TonicDataPointValue {
            number_data_point_value_from_i64(self)
        }
        
        fn to_exemplar_value(self) -> TonicExemplarValue {
            exemplar_value_from_i64(self)
        }
    }

    impl Numeric for f64 {
        fn into_f64(self) -> f64 {
            self
        }
        
        fn to_data_point_value(self) -> TonicDataPointValue {
            number_data_point_value_from_f64(self)
        }
        
        fn to_exemplar_value(self) -> TonicExemplarValue {
            exemplar_value_from_f64(self)
        }
    }

    pub fn sdk_histogram_to_tonic_histogram<T>(hist: &SdkHistogram<T>) -> TonicHistogram
    where
        T: Numeric,
    {
        TonicHistogram {
            data_points: hist
                .data_points()
                .map(|dp| TonicHistogramDataPoint {
                    attributes: dp.attributes().map(|kv| key_value_from_otel_key_value(kv)).collect(),
                    start_time_unix_nano: to_nanos(hist.start_time()),
                    time_unix_nano: to_nanos(hist.time()),
                    count: dp.count(),
                    sum: Some(dp.sum().into_f64()),
                    bucket_counts: dp.bucket_counts().collect(),
                    explicit_bounds: dp.bounds().collect(),
                    exemplars: dp.exemplars().map(|ex| sdk_exemplar_to_tonic_exemplar(ex)).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    min: dp.min().map(Numeric::into_f64),
                    max: dp.max().map(Numeric::into_f64),
                })
                .collect(),
            aggregation_temporality: temporality_to_aggregation_temporality(hist.temporality()).into(),
        }
    }

    pub fn sdk_exponential_histogram_to_tonic_exponential_histogram<T>(hist: &SdkExponentialHistogram<T>) -> TonicExponentialHistogram
    where
        T: Numeric,
    {
        TonicExponentialHistogram {
            data_points: hist
                .data_points()
                .map(|dp| TonicExponentialHistogramDataPoint {
                    attributes: dp.attributes().map(|kv| key_value_from_otel_key_value(kv)).collect(),
                    start_time_unix_nano: to_nanos(hist.start_time()),
                    time_unix_nano: to_nanos(hist.time()),
                    count: dp.count() as u64,
                    sum: Some(dp.sum().into_f64()),
                    scale: dp.scale().into(),
                    zero_count: dp.zero_count(),
                    positive: Some(TonicBuckets {
                        offset: dp.positive_bucket().offset(),
                        bucket_counts: dp.positive_bucket().counts().collect(),
                    }),
                    negative: Some(TonicBuckets {
                        offset: dp.negative_bucket().offset(),
                        bucket_counts: dp.negative_bucket().counts().collect(),
                    }),
                    flags: TonicDataPointFlags::default() as u32,
                    exemplars: dp.exemplars().map(|ex| sdk_exemplar_to_tonic_exemplar(ex)).collect(),
                    min: dp.min().map(Numeric::into_f64),
                    max: dp.max().map(Numeric::into_f64),
                    zero_threshold: dp.zero_threshold(),
                })
                .collect(),
            aggregation_temporality: temporality_to_aggregation_temporality(hist.temporality()).into(),
        }
    }

    pub fn sdk_sum_to_tonic_sum<T>(sum: &SdkSum<T>) -> TonicSum
    where
        T: Numeric + Debug,
    {
        TonicSum {
            data_points: sum
                .data_points()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes().map(|kv| key_value_from_otel_key_value(kv)).collect(),
                    start_time_unix_nano: to_nanos(sum.start_time()),
                    time_unix_nano: to_nanos(sum.time()),
                    exemplars: dp.exemplars().map(|ex| sdk_exemplar_to_tonic_exemplar(ex)).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value().to_data_point_value()),
                })
                .collect(),
            aggregation_temporality: temporality_to_aggregation_temporality(sum.temporality()).into(),
            is_monotonic: sum.is_monotonic(),
        }
    }

    pub fn sdk_gauge_to_tonic_gauge<T>(gauge: &SdkGauge<T>) -> TonicGauge
    where
        T: Numeric + Debug,
    {
        TonicGauge {
            data_points: gauge
                .data_points()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes().map(|kv| key_value_from_otel_key_value(kv)).collect(),
                    start_time_unix_nano: gauge.start_time().map(to_nanos).unwrap_or_default(),
                    time_unix_nano: to_nanos(gauge.time()),
                    exemplars: dp.exemplars().map(|ex| sdk_exemplar_to_tonic_exemplar(ex)).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value().to_data_point_value()),
                })
                .collect(),
        }
    }

    pub fn sdk_exemplar_to_tonic_exemplar<T>(ex: &SdkExemplar<T>) -> TonicExemplar
    where
        T: Numeric,
    {
        TonicExemplar {
            filtered_attributes: ex
                .filtered_attributes()
                .map(|kv| key_value_from_key_value_ref((&kv.key, &kv.value)))
                .collect(),
            time_unix_nano: to_nanos(ex.time()),
            span_id: ex.span_id().into(),
            trace_id: ex.trace_id().into(),
            value: Some(ex.value.to_exemplar_value()),
        }
    }
}
