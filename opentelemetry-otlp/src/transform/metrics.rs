#[cfg(any(feature = "grpc-tonic", feature = "http-proto"))]
pub(crate) mod tonic {
    use std::any::Any;
    use std::fmt;

    use opentelemetry_api::{global, metrics::MetricsError};
    use opentelemetry_proto::tonic::common::v1::InstrumentationScope as TonicInstrumentationScope;
    use opentelemetry_proto::tonic::resource::v1::Resource as TonicResource;
    use opentelemetry_proto::tonic::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        metrics::v1::{
            exemplar::Value as TonicExemplarValue, metric::Data as TonicMetricData,
            number_data_point::Value as TonicDataPointValue,
            AggregationTemporality as TonicTemporality, DataPointFlags as TonicDataPointFlags,
            Exemplar as TonicExemplar, Gauge as TonicGauge, Histogram as TonicHistogram,
            HistogramDataPoint as TonicHistogramDataPoint, Metric as TonicMetric,
            NumberDataPoint as TonicNumberDataPoint, ResourceMetrics as TonicResourceMetrics,
            ScopeMetrics as TonicScopeMetrics, Sum as TonicSum,
        },
    };
    use opentelemetry_sdk::metrics::data::{
        self, Exemplar as SdkExemplar, Gauge as SdkGauge, Histogram as SdkHistogram,
        Metric as SdkMetric, ScopeMetrics as SdkScopeMetrics, Sum as SdkSum,
    };
    use opentelemetry_sdk::Resource as SdkResource;

    use crate::to_nanos;

    pub(crate) fn sink(metrics: &data::ResourceMetrics) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![TonicResourceMetrics {
                resource: transform_resource(&metrics.resource),
                scope_metrics: transform_scope_metrics(&metrics.scope_metrics),
                schema_url: metrics
                    .resource
                    .schema_url()
                    .map(Into::into)
                    .unwrap_or_default(),
            }],
        }
    }

    fn transform_resource(r: &SdkResource) -> Option<TonicResource> {
        if r.is_empty() {
            return None;
        }

        Some(TonicResource {
            attributes: r.iter().map(Into::into).collect(),
            dropped_attributes_count: 0,
        })
    }

    fn transform_scope_metrics(sms: &[SdkScopeMetrics]) -> Vec<TonicScopeMetrics> {
        sms.iter()
            .map(|sm| TonicScopeMetrics {
                scope: Some(TonicInstrumentationScope::from(&sm.scope)),
                metrics: transform_metrics(&sm.metrics),
                schema_url: sm
                    .scope
                    .schema_url
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            })
            .collect()
    }

    fn transform_metrics(metrics: &[SdkMetric]) -> Vec<TonicMetric> {
        metrics
            .iter()
            .map(|metric| TonicMetric {
                name: metric.name.to_string(),
                description: metric.description.to_string(),
                unit: metric.unit.as_str().to_string(),
                data: transform_data(metric.data.as_any()),
            })
            .collect()
    }

    fn transform_data(data: &dyn Any) -> Option<TonicMetricData> {
        if let Some(hist) = data.downcast_ref::<SdkHistogram<i64>>() {
            Some(TonicMetricData::Histogram(transform_histogram(hist)))
        } else if let Some(hist) = data.downcast_ref::<SdkHistogram<u64>>() {
            Some(TonicMetricData::Histogram(transform_histogram(hist)))
        } else if let Some(hist) = data.downcast_ref::<SdkHistogram<f64>>() {
            Some(TonicMetricData::Histogram(transform_histogram(hist)))
        } else if let Some(sum) = data.downcast_ref::<SdkSum<u64>>() {
            Some(TonicMetricData::Sum(transform_sum(sum)))
        } else if let Some(sum) = data.downcast_ref::<SdkSum<i64>>() {
            Some(TonicMetricData::Sum(transform_sum(sum)))
        } else if let Some(sum) = data.downcast_ref::<SdkSum<f64>>() {
            Some(TonicMetricData::Sum(transform_sum(sum)))
        } else if let Some(gauge) = data.downcast_ref::<SdkGauge<u64>>() {
            Some(TonicMetricData::Gauge(transform_gauge(gauge)))
        } else if let Some(gauge) = data.downcast_ref::<SdkGauge<i64>>() {
            Some(TonicMetricData::Gauge(transform_gauge(gauge)))
        } else if let Some(gauge) = data.downcast_ref::<SdkGauge<f64>>() {
            Some(TonicMetricData::Gauge(transform_gauge(gauge)))
        } else {
            global::handle_error(MetricsError::Other("unknown aggregator".into()));
            None
        }
    }

    fn transform_histogram<T: Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy>(
        hist: &SdkHistogram<T>,
    ) -> TonicHistogram {
        TonicHistogram {
            data_points: hist
                .data_points
                .iter()
                .map(|dp| TonicHistogramDataPoint {
                    attributes: dp.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: to_nanos(dp.start_time),
                    time_unix_nano: to_nanos(dp.time),
                    count: dp.count,
                    sum: Some(dp.sum),
                    bucket_counts: dp.bucket_counts.clone(),
                    explicit_bounds: dp.bounds.clone(),
                    exemplars: dp.exemplars.iter().map(transform_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    min: dp.min,
                    max: dp.max,
                })
                .collect(),
            aggregation_temporality: TonicTemporality::from(hist.temporality).into(),
        }
    }

    fn transform_sum<
        T: fmt::Debug + Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy,
    >(
        sum: &SdkSum<T>,
    ) -> TonicSum {
        TonicSum {
            data_points: sum
                .data_points
                .iter()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: dp.start_time.map(to_nanos).unwrap_or_default(),
                    time_unix_nano: dp.time.map(to_nanos).unwrap_or_default(),
                    exemplars: dp.exemplars.iter().map(transform_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value.into()),
                })
                .collect(),
            aggregation_temporality: TonicTemporality::from(sum.temporality).into(),
            is_monotonic: sum.is_monotonic,
        }
    }

    fn transform_gauge<
        T: fmt::Debug + Into<TonicExemplarValue> + Into<TonicDataPointValue> + Copy,
    >(
        gauge: &SdkGauge<T>,
    ) -> TonicGauge {
        TonicGauge {
            data_points: gauge
                .data_points
                .iter()
                .map(|dp| TonicNumberDataPoint {
                    attributes: dp.attributes.iter().map(Into::into).collect(),
                    start_time_unix_nano: dp.start_time.map(to_nanos).unwrap_or_default(),
                    time_unix_nano: dp.time.map(to_nanos).unwrap_or_default(),
                    exemplars: dp.exemplars.iter().map(transform_exemplar).collect(),
                    flags: TonicDataPointFlags::default() as u32,
                    value: Some(dp.value.into()),
                })
                .collect(),
        }
    }

    fn transform_exemplar<T: Into<TonicExemplarValue> + Copy>(
        ex: &SdkExemplar<T>,
    ) -> TonicExemplar {
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
