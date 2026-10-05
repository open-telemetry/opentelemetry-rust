#![cfg(all(
    unix,
    any(feature = "reqwest-client", feature = "reqwest-blocking-client")
))]

use integration_test_runner::fake_otlp_http::{FakeOtlpHttpEndpoint, ScriptedHttpResponse};
use opentelemetry::{metrics::MeterProvider, InstrumentationScope, Key, KeyValue};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_proto::tonic::{
    collector::metrics::v1::ExportMetricsServiceRequest,
    common::v1::{any_value, AnyValue, KeyValue as ProtoKeyValue},
    metrics::v1::{exemplar, metric, number_data_point, AggregationTemporality},
};
use opentelemetry_sdk::{
    error::OTelSdkError,
    metrics::{
        data::{
            Exemplar, ExponentialBucket, ExponentialHistogram, ExponentialHistogramDataPoint,
            Gauge, GaugeDataPoint, Histogram, HistogramDataPoint, Metric, MetricData, ScopeMetrics,
            Sum, SumDataPoint,
        },
        MetricProducer, SdkMeterProvider, Temporality,
    },
    Resource,
};
use prost::Message;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

#[derive(Debug)]
struct ExternalMetrics(Arc<AtomicUsize>);

impl MetricProducer for ExternalMetrics {
    fn produce(&self, resource: &Resource) -> Result<Vec<ScopeMetrics>, OTelSdkError> {
        assert_eq!(
            resource.get(&Key::new("service.name")),
            Some("producer-e2e".into())
        );
        self.0.fetch_add(1, Ordering::Relaxed);
        let start = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
        let time = start + Duration::from_secs(5);
        let attributes = vec![KeyValue::new("source", "external")];
        let exemplar = Exemplar::builder(1.5, time)
            .with_filtered_attributes(vec![KeyValue::new("request.id", "abc")])
            .with_span_id([1; 8])
            .with_trace_id([2; 16])
            .build();

        let gauge = Gauge::builder(
            vec![GaugeDataPoint::builder(23.5)
                .with_attributes(attributes.clone())
                .build()],
            time,
        )
        .with_start_time(start)
        .build();
        let sum = Sum::new(
            vec![SumDataPoint::builder(-7_i64)
                .with_attributes(attributes.clone())
                .build()],
            Temporality::Cumulative,
            false,
            start,
            time,
        );
        let histogram = Histogram::new(
            vec![
                HistogramDataPoint::builder(3, 12_u64, vec![2.0, 5.0], vec![1, 1, 1])
                    .with_attributes(attributes.clone())
                    .with_min(1)
                    .with_max(7)
                    .build(),
            ],
            Temporality::Cumulative,
            start,
            time,
        );
        let exponential = ExponentialHistogram::new(
            vec![ExponentialHistogramDataPoint::builder(
                3,
                0.75,
                0,
                1,
                ExponentialBucket::new(0, vec![1]),
                ExponentialBucket::new(-1, vec![1]),
            )
            .with_attributes(attributes)
            .with_min(-0.75)
            .with_max(1.5)
            .with_zero_threshold(0.01)
            .with_exemplars(vec![exemplar])
            .build()],
            Temporality::Cumulative,
            start,
            time,
        );
        Ok(vec![ScopeMetrics::builder()
            .with_scope(
                InstrumentationScope::builder("external-bridge")
                    .with_version("1.0")
                    .with_schema_url("https://example.com/schema")
                    .with_attributes(vec![KeyValue::new("bridge", "test")])
                    .build(),
            )
            .with_metrics(vec![
                Metric::builder("external.temperature", MetricData::from(gauge).into())
                    .with_description("External temperature")
                    .with_unit("Cel")
                    .build(),
                Metric::builder("external.balance", MetricData::from(sum).into()).build(),
                Metric::builder("external.histogram", MetricData::from(histogram).into()).build(),
                Metric::builder("external.exponential", MetricData::from(exponential).into())
                    .build(),
            ])
            .build()])
    }
}

fn attribute(key: &str, value: &str) -> ProtoKeyValue {
    ProtoKeyValue {
        key: key.into(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.into())),
        }),
        ..Default::default()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn external_and_sdk_metrics_reach_otlp_http() {
    let endpoint =
        FakeOtlpHttpEndpoint::start((0..3).map(|_| ScriptedHttpResponse::new(200)).collect())
            .await
            .unwrap();
    let url = endpoint.endpoint("/v1/metrics");
    let calls = Arc::new(AtomicUsize::new(0));
    let producer = ExternalMetrics(calls.clone());

    tokio::task::spawn_blocking(move || {
        let exporter = MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(url)
            .with_timeout(Duration::from_secs(5))
            .with_temporality(Temporality::Cumulative)
            .build()
            .unwrap();
        #[cfg(feature = "reqwest-client")]
        let reader =
            opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder(
                exporter,
                opentelemetry_sdk::runtime::Tokio,
            );
        #[cfg(not(feature = "reqwest-client"))]
        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter);

        let provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder_empty()
                    .with_service_name("producer-e2e")
                    .build(),
            )
            .with_reader(
                reader
                    .with_interval(Duration::from_secs(3600))
                    .with_producer(producer)
                    .build(),
            )
            .build();
        provider
            .meter("application")
            .u64_counter("sdk.requests")
            .build()
            .add(9, &[]);
        provider.force_flush().unwrap();
        provider.force_flush().unwrap();
        provider.shutdown().unwrap();
        assert!(provider.force_flush().is_err());
    })
    .await
    .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 3);
    let requests = endpoint.requests();
    assert_eq!(requests.len(), 3);
    for request in requests {
        assert_eq!(request.request_line, "POST /v1/metrics HTTP/1.1");
        assert_eq!(request.headers["content-type"], "application/x-protobuf");
        let decoded = ExportMetricsServiceRequest::decode(request.body.as_slice()).unwrap();
        assert_eq!(decoded.resource_metrics.len(), 1);
        let rm = &decoded.resource_metrics[0];
        assert_eq!(
            rm.resource.as_ref().unwrap().attributes,
            vec![attribute("service.name", "producer-e2e")]
        );
        assert_eq!(rm.scope_metrics.len(), 2);
        let sdk = rm
            .scope_metrics
            .iter()
            .find(|s| s.scope.as_ref().unwrap().name == "application")
            .unwrap();
        assert_eq!(sdk.metrics.len(), 1);
        assert_eq!(sdk.metrics[0].name, "sdk.requests");
        let Some(metric::Data::Sum(sdk_sum)) = &sdk.metrics[0].data else {
            panic!("expected SDK sum")
        };
        assert_eq!(
            sdk_sum.data_points[0].value,
            Some(number_data_point::Value::AsInt(9))
        );

        let external = rm
            .scope_metrics
            .iter()
            .find(|s| s.scope.as_ref().unwrap().name == "external-bridge")
            .unwrap();
        assert_eq!(external.scope.as_ref().unwrap().version, "1.0");
        assert_eq!(
            external.scope.as_ref().unwrap().attributes,
            vec![attribute("bridge", "test")]
        );
        assert_eq!(external.schema_url, "https://example.com/schema");
        assert_eq!(external.metrics.len(), 4);
        let names: Vec<_> = external.metrics.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "external.temperature",
                "external.balance",
                "external.histogram",
                "external.exponential"
            ]
        );
        assert_eq!(external.metrics[0].description, "External temperature");
        assert_eq!(external.metrics[0].unit, "Cel");
        let Some(metric::Data::Gauge(gauge)) = &external.metrics[0].data else {
            panic!("expected gauge")
        };
        assert_eq!(gauge.data_points.len(), 1);
        let point = &gauge.data_points[0];
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(23.5)));
        assert_eq!(
            (point.start_time_unix_nano, point.time_unix_nano),
            (10_000_000_000, 15_000_000_000)
        );
        assert_eq!(point.attributes, vec![attribute("source", "external")]);

        let Some(metric::Data::Sum(sum)) = &external.metrics[1].data else {
            panic!("expected sum")
        };
        assert_eq!(
            sum.aggregation_temporality(),
            AggregationTemporality::Cumulative
        );
        assert!(!sum.is_monotonic);
        assert_eq!(sum.data_points.len(), 1);
        assert_eq!(
            sum.data_points[0].value,
            Some(number_data_point::Value::AsInt(-7))
        );
        assert_eq!(sum.data_points[0].attributes, point.attributes);
        assert_eq!(
            (
                sum.data_points[0].start_time_unix_nano,
                sum.data_points[0].time_unix_nano
            ),
            (10_000_000_000, 15_000_000_000)
        );

        let Some(metric::Data::Histogram(histogram)) = &external.metrics[2].data else {
            panic!("expected histogram")
        };
        assert_eq!(
            histogram.aggregation_temporality(),
            AggregationTemporality::Cumulative
        );
        assert_eq!(histogram.data_points.len(), 1);
        let histogram = &histogram.data_points[0];
        assert_eq!(histogram.count, 3);
        assert_eq!(histogram.sum, Some(12.0));
        assert_eq!((histogram.min, histogram.max), (Some(1.0), Some(7.0)));
        assert_eq!(histogram.explicit_bounds, vec![2.0, 5.0]);
        assert_eq!(histogram.bucket_counts, vec![1, 1, 1]);
        assert_eq!(histogram.attributes, point.attributes);
        assert_eq!(
            (histogram.start_time_unix_nano, histogram.time_unix_nano),
            (10_000_000_000, 15_000_000_000)
        );

        let Some(metric::Data::ExponentialHistogram(exponential)) = &external.metrics[3].data
        else {
            panic!("expected exponential histogram")
        };
        assert_eq!(
            exponential.aggregation_temporality(),
            AggregationTemporality::Cumulative
        );
        assert_eq!(exponential.data_points.len(), 1);
        let exponential = &exponential.data_points[0];
        assert_eq!(exponential.count, 3);
        assert_eq!(exponential.sum, Some(0.75));
        assert_eq!((exponential.min, exponential.max), (Some(-0.75), Some(1.5)));
        assert_eq!(exponential.scale, 0);
        assert_eq!(exponential.zero_count, 1);
        assert_eq!(exponential.zero_threshold, 0.01);
        let positive = exponential.positive.as_ref().unwrap();
        let negative = exponential.negative.as_ref().unwrap();
        assert_eq!((positive.offset, &positive.bucket_counts), (0, &vec![1]));
        assert_eq!((negative.offset, &negative.bucket_counts), (-1, &vec![1]));
        assert_eq!(exponential.attributes, point.attributes);
        assert_eq!(
            (exponential.start_time_unix_nano, exponential.time_unix_nano),
            (10_000_000_000, 15_000_000_000)
        );
        assert_eq!(exponential.exemplars.len(), 1);
        let exemplar = &exponential.exemplars[0];
        assert_eq!(exemplar.value, Some(exemplar::Value::AsDouble(1.5)));
        assert_eq!(exemplar.time_unix_nano, 15_000_000_000);
        assert_eq!(exemplar.span_id, [1; 8]);
        assert_eq!(exemplar.trace_id, [2; 16]);
        assert_eq!(
            exemplar.filtered_attributes,
            vec![attribute("request.id", "abc")]
        );
    }
}
