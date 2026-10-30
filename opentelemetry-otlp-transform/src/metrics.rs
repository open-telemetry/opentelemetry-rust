use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_sdk::metrics::data::ResourceMetrics;

/// Converts collected metrics into an OTLP export request.
pub fn export_request(metrics: &ResourceMetrics) -> ExportMetricsServiceRequest {
    tonic::resource_metrics_to_export_request(metrics)
}

// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
pub(crate) mod tonic {
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

    use crate::common::{to_nanos, tonic::value_to_any_value};
    use opentelemetry_proto::tonic::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            exemplar, exponential_histogram_data_point::Buckets as TonicBuckets,
            metric::Data as TonicMetricData, number_data_point, AggregationTemporality,
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

    pub(crate) fn u64_to_exemplar_value(value: u64) -> exemplar::Value {
        exemplar::Value::AsInt(i64::try_from(value).unwrap_or_default())
    }

    pub(crate) fn i64_to_exemplar_value(value: i64) -> exemplar::Value {
        exemplar::Value::AsInt(value)
    }

    pub(crate) fn f64_to_exemplar_value(value: f64) -> exemplar::Value {
        exemplar::Value::AsDouble(value)
    }

    pub(crate) fn u64_to_data_point_value(value: u64) -> number_data_point::Value {
        number_data_point::Value::AsInt(i64::try_from(value).unwrap_or_default())
    }

    pub(crate) fn i64_to_data_point_value(value: i64) -> number_data_point::Value {
        number_data_point::Value::AsInt(value)
    }

    pub(crate) fn f64_to_data_point_value(value: f64) -> number_data_point::Value {
        number_data_point::Value::AsDouble(value)
    }

    pub(crate) fn key_value_ref_to_proto(kv: (&Key, &Value)) -> KeyValue {
        KeyValue {
            key: kv.0.to_string(),
            value: Some(value_to_any_value(kv.1.clone())),
            key_strindex: 0,
        }
    }

    pub(crate) fn api_key_value_ref_to_proto(kv: &opentelemetry::KeyValue) -> KeyValue {
        KeyValue {
            key: kv.key.to_string(),
            value: Some(value_to_any_value(kv.value.clone())),
            key_strindex: 0,
        }
    }

    pub(crate) fn temporality_to_proto(temporality: Temporality) -> AggregationTemporality {
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

    pub(crate) fn resource_metrics_to_export_request(
        rm: &ResourceMetrics,
    ) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![TonicResourceMetrics {
                resource: Some(sdk_resource_to_proto(rm.resource())),
                scope_metrics: rm.scope_metrics().map(scope_metrics_to_proto).collect(),
                schema_url: rm
                    .resource()
                    .schema_url()
                    .map(Into::into)
                    .unwrap_or_default(),
            }],
        }
    }

    pub(crate) fn sdk_resource_to_proto(resource: &SdkResource) -> TonicResource {
        TonicResource {
            attributes: resource.iter().map(key_value_ref_to_proto).collect(),
            dropped_attributes_count: 0,
            entity_refs: vec![], // internal and currently unused
        }
    }

    pub(crate) fn scope_metrics_to_proto(sm: &SdkScopeMetrics) -> TonicScopeMetrics {
        TonicScopeMetrics {
            scope: Some(crate::common::tonic::instrumentation_scope_ref_to_proto(
                sm.scope(),
                None,
            )),
            metrics: sm.metrics().map(metric_to_proto).collect(),
            schema_url: sm
                .scope()
                .schema_url()
                .map(ToOwned::to_owned)
                .unwrap_or_default(),
        }
    }

    pub(crate) fn metric_to_proto(metric: &SdkMetric) -> TonicMetric {
        TonicMetric {
            name: metric.name().to_string(),
            description: metric.description().to_string(),
            unit: metric.unit().to_string(),
            metadata: vec![], // internal and currently unused
            data: Some(match metric.data() {
                AggregatedMetrics::F64(data) => metric_data_to_proto(data),
                AggregatedMetrics::U64(data) => metric_data_to_proto(data),
                AggregatedMetrics::I64(data) => metric_data_to_proto(data),
            }),
        }
    }

    pub(crate) fn metric_data_to_proto<T>(data: &MetricData<T>) -> TonicMetricData
    where
        T: Numeric + Debug,
    {
        match data {
            MetricData::Gauge(gauge) => TonicMetricData::Gauge(gauge_to_proto(gauge)),
            MetricData::Sum(sum) => TonicMetricData::Sum(sum_to_proto(sum)),
            MetricData::Histogram(hist) => TonicMetricData::Histogram(histogram_to_proto(hist)),
            MetricData::ExponentialHistogram(hist) => {
                TonicMetricData::ExponentialHistogram(exponential_histogram_to_proto(hist))
            }
        }
    }

    pub(crate) trait Numeric: Copy {
        fn to_exemplar_value(self) -> exemplar::Value;
        fn to_data_point_value(self) -> number_data_point::Value;
        // lossy at large values for u64 and i64 but otlp histograms only handle float values
        fn into_f64(self) -> f64;
    }

    impl Numeric for u64 {
        fn to_exemplar_value(self) -> exemplar::Value {
            u64_to_exemplar_value(self)
        }
        fn to_data_point_value(self) -> number_data_point::Value {
            u64_to_data_point_value(self)
        }
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    impl Numeric for i64 {
        fn to_exemplar_value(self) -> exemplar::Value {
            i64_to_exemplar_value(self)
        }
        fn to_data_point_value(self) -> number_data_point::Value {
            i64_to_data_point_value(self)
        }
        fn into_f64(self) -> f64 {
            self as f64
        }
    }

    impl Numeric for f64 {
        fn to_exemplar_value(self) -> exemplar::Value {
            f64_to_exemplar_value(self)
        }
        fn to_data_point_value(self) -> number_data_point::Value {
            f64_to_data_point_value(self)
        }
        fn into_f64(self) -> f64 {
            self
        }
    }

    pub(crate) fn histogram_to_proto<T>(hist: &SdkHistogram<T>) -> TonicHistogram
    where
        T: Numeric,
    {
        TonicHistogram {
            data_points: hist
                .data_points()
                .map(|dp| TonicHistogramDataPoint {
                    attributes: dp.attributes().map(api_key_value_ref_to_proto).collect(),
                    start_time_unix_nano: to_nanos(hist.start_time()),
                    time_unix_nano: to_nanos(hist.time()),
                    count: dp.count(),
                    sum: Some(dp.sum().into_f64()),
                    bucket_counts: dp.bucket_counts().collect(),
                    explicit_bounds: dp.bounds().collect(),
                    exemplars: dp.exemplars().map(exemplar_to_proto).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    min: dp.min().map(Numeric::into_f64),
                    max: dp.max().map(Numeric::into_f64),
                })
                .collect(),
            aggregation_temporality: temporality_to_proto(hist.temporality()).into(),
        }
    }

    pub(crate) fn exponential_histogram_to_proto<T>(
        hist: &SdkExponentialHistogram<T>,
    ) -> TonicExponentialHistogram
    where
        T: Numeric,
    {
        TonicExponentialHistogram {
            data_points: hist
                .data_points()
                .map(|dp| TonicExponentialHistogramDataPoint {
                    attributes: dp.attributes().map(api_key_value_ref_to_proto).collect(),
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
                    exemplars: dp.exemplars().map(exemplar_to_proto).collect(),
                    min: dp.min().map(Numeric::into_f64),
                    max: dp.max().map(Numeric::into_f64),
                    zero_threshold: dp.zero_threshold(),
                })
                .collect(),
            aggregation_temporality: temporality_to_proto(hist.temporality()).into(),
        }
    }

    pub(crate) fn sum_to_proto<T>(sum: &SdkSum<T>) -> TonicSum
    where
        T: Numeric + Debug,
    {
        TonicSum {
            data_points: sum
                .data_points()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes().map(api_key_value_ref_to_proto).collect(),
                    start_time_unix_nano: to_nanos(sum.start_time()),
                    time_unix_nano: to_nanos(sum.time()),
                    exemplars: dp.exemplars().map(exemplar_to_proto).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value().to_data_point_value()),
                })
                .collect(),
            aggregation_temporality: temporality_to_proto(sum.temporality()).into(),
            is_monotonic: sum.is_monotonic(),
        }
    }

    pub(crate) fn gauge_to_proto<T>(gauge: &SdkGauge<T>) -> TonicGauge
    where
        T: Numeric + Debug,
    {
        TonicGauge {
            data_points: gauge
                .data_points()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes().map(api_key_value_ref_to_proto).collect(),
                    start_time_unix_nano: gauge.start_time().map(to_nanos).unwrap_or_default(),
                    time_unix_nano: to_nanos(gauge.time()),
                    exemplars: dp.exemplars().map(exemplar_to_proto).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value().to_data_point_value()),
                })
                .collect(),
        }
    }

    pub(crate) fn exemplar_to_proto<T>(ex: &SdkExemplar<T>) -> TonicExemplar
    where
        T: Numeric,
    {
        TonicExemplar {
            filtered_attributes: ex
                .filtered_attributes()
                .map(|kv| key_value_ref_to_proto((&kv.key, &kv.value)))
                .collect(),
            time_unix_nano: to_nanos(ex.time()),
            span_id: ex.span_id().into(),
            trace_id: ex.trace_id().into(),
            value: Some(ex.value.to_exemplar_value()),
        }
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry::{InstrumentationScope, KeyValue};
    use opentelemetry_proto::tonic::metrics::v1::{
        exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as NumberValue,
        AggregationTemporality,
    };
    use opentelemetry_sdk::metrics::{
        Aggregation, InMemoryMetricExporter, SdkMeterProvider, Stream,
    };
    use opentelemetry_sdk::Resource;

    #[test]
    fn numeric_values_and_temporalities_convert_to_otlp() {
        use super::tonic::{
            f64_to_data_point_value, f64_to_exemplar_value, i64_to_data_point_value,
            i64_to_exemplar_value, temporality_to_proto, u64_to_data_point_value,
            u64_to_exemplar_value,
        };
        use opentelemetry_sdk::metrics::Temporality;

        assert_eq!(u64_to_exemplar_value(7), ExemplarValue::AsInt(7));
        assert_eq!(i64_to_exemplar_value(-7), ExemplarValue::AsInt(-7));
        assert_eq!(f64_to_exemplar_value(7.5), ExemplarValue::AsDouble(7.5));
        assert_eq!(u64_to_data_point_value(8), NumberValue::AsInt(8));
        assert_eq!(i64_to_data_point_value(-8), NumberValue::AsInt(-8));
        assert_eq!(f64_to_data_point_value(8.5), NumberValue::AsDouble(8.5));
        assert_eq!(
            temporality_to_proto(Temporality::Delta),
            AggregationTemporality::Delta
        );
        assert_eq!(
            temporality_to_proto(Temporality::LowMemory),
            AggregationTemporality::Cumulative
        );
    }

    #[test]
    fn public_export_request_converts_all_metric_aggregations() {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_schema_url(
                        [KeyValue::new("service.name", "checkout")],
                        "https://example.com/resource",
                    )
                    .build(),
            )
            .with_view(|instrument| {
                (instrument.name() == "exponential").then(|| {
                    Stream::builder()
                        .with_aggregation(Aggregation::Base2ExponentialHistogram {
                            max_size: 160,
                            max_scale: 20,
                            record_min_max: true,
                        })
                        .build()
                        .expect("exponential histogram stream should be valid")
                })
            })
            .with_periodic_exporter(exporter.clone())
            .build();
        let scope = InstrumentationScope::builder("test-meter")
            .with_version("1.2.3")
            .with_schema_url("https://example.com/scope")
            .with_attributes([KeyValue::new("scope.attribute", "value")])
            .build();
        let meter = provider.meter_with_scope(scope);
        let attributes = [KeyValue::new("route", "/checkout")];

        meter
            .u64_gauge("gauge")
            .with_description("current value")
            .with_unit("1")
            .build()
            .record(7, &attributes);
        meter.u64_counter("sum").build().add(3, &attributes);
        meter
            .i64_up_down_counter("signed-sum")
            .build()
            .add(-2, &attributes);
        let histogram = meter.f64_histogram("histogram").build();
        histogram.record(1.0, &attributes);
        histogram.record(2.0, &attributes);
        meter.u64_counter("exponential").build().add(4, &attributes);

        provider.force_flush().unwrap();
        let collected = exporter.get_finished_metrics().unwrap();
        assert_eq!(collected.len(), 1);

        let request = super::export_request(&collected[0]);
        let resource_metrics = &request.resource_metrics[0];
        assert_eq!(resource_metrics.schema_url, "https://example.com/resource");
        assert_eq!(
            resource_metrics.resource.as_ref().unwrap().attributes[0].key,
            "service.name"
        );

        let scope_metrics = &resource_metrics.scope_metrics[0];
        assert_eq!(scope_metrics.schema_url, "https://example.com/scope");
        let exported_scope = scope_metrics.scope.as_ref().unwrap();
        assert_eq!(exported_scope.name, "test-meter");
        assert_eq!(exported_scope.version, "1.2.3");
        assert_eq!(exported_scope.attributes[0].key, "scope.attribute");

        let metric = |name: &str| {
            scope_metrics
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .unwrap_or_else(|| panic!("missing metric {name}"))
        };

        let gauge = metric("gauge");
        assert_eq!(gauge.description, "current value");
        assert_eq!(gauge.unit, "1");
        let Some(Data::Gauge(gauge)) = &gauge.data else {
            panic!("expected gauge data");
        };
        assert_eq!(gauge.data_points.len(), 1);
        assert_eq!(gauge.data_points[0].value, Some(NumberValue::AsInt(7)));
        assert_eq!(gauge.data_points[0].attributes[0].key, "route");
        assert!(gauge.data_points[0].exemplars.is_empty());

        let Some(Data::Sum(sum)) = &metric("sum").data else {
            panic!("expected sum data");
        };
        assert!(sum.is_monotonic);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Cumulative as i32
        );
        assert_eq!(sum.data_points[0].value, Some(NumberValue::AsInt(3)));
        assert!(sum.data_points[0].exemplars.is_empty());

        let Some(Data::Sum(sum)) = &metric("signed-sum").data else {
            panic!("expected signed sum data");
        };
        assert!(!sum.is_monotonic);
        assert_eq!(sum.data_points[0].value, Some(NumberValue::AsInt(-2)));

        let Some(Data::Histogram(histogram)) = &metric("histogram").data else {
            panic!("expected histogram data");
        };
        let point = &histogram.data_points[0];
        assert_eq!(point.count, 2);
        assert_eq!(point.sum, Some(3.0));
        assert_eq!(point.min, Some(1.0));
        assert_eq!(point.max, Some(2.0));
        assert!(point.exemplars.is_empty());

        let Some(Data::ExponentialHistogram(histogram)) = &metric("exponential").data else {
            panic!("expected exponential histogram data");
        };
        let point = &histogram.data_points[0];
        assert_eq!(point.count, 1);
        assert_eq!(point.sum, Some(4.0));
        assert_eq!(point.min, Some(4.0));
        assert_eq!(point.max, Some(4.0));
        assert!(point.positive.is_some());
        assert!(point.exemplars.is_empty());
    }
}
