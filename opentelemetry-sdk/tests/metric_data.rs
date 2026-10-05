#![cfg(feature = "metrics")]

use opentelemetry::{InstrumentationScope, Key, KeyValue};
use opentelemetry_sdk::{
    metrics::{data::*, Temporality},
    Resource,
};
use std::time::{Duration, SystemTime};

fn times() -> (SystemTime, SystemTime) {
    let start = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
    (start, start + Duration::from_secs(5))
}

#[test]
fn gauge_and_metric_containers_preserve_public_inputs() {
    let (start, time) = times();
    let attributes = vec![KeyValue::new("region", "west")];
    let exemplar = Exemplar::builder(42.5, time)
        .with_filtered_attributes(attributes.clone())
        .with_span_id([1; 8])
        .with_trace_id([2; 16])
        .build();
    assert_eq!(exemplar.value, 42.5);
    assert_eq!(exemplar.time(), time);
    assert_eq!(
        exemplar.filtered_attributes().collect::<Vec<_>>(),
        vec![&attributes[0]]
    );
    assert_eq!(exemplar.span_id(), &[1; 8]);
    assert_eq!(exemplar.trace_id(), &[2; 16]);

    let point = GaugeDataPoint::builder(42.5)
        .with_attributes(vec![KeyValue::new("replaced", true)])
        .with_attributes(attributes.clone())
        .with_exemplars(vec![exemplar.clone()])
        .build();
    assert_eq!(point.value(), 42.5);
    assert_eq!(point.attributes().collect::<Vec<_>>(), vec![&attributes[0]]);
    assert_eq!(point.exemplars().collect::<Vec<_>>(), vec![&exemplar]);

    let gauge = Gauge::builder(vec![point], time)
        .with_start_time(start)
        .build();
    assert_eq!(gauge.start_time(), Some(start));
    assert_eq!(gauge.time(), time);
    assert_eq!(gauge.data_points().count(), 1);
    let metric = Metric::builder(
        String::from("external.temperature"),
        MetricData::from(gauge).into(),
    )
    .with_description(String::from("External temperature"))
    .with_unit("Cel")
    .build();
    assert_eq!(metric.name(), "external.temperature");
    assert_eq!(metric.description(), "External temperature");
    assert_eq!(metric.unit(), "Cel");
    assert!(matches!(
        metric.data(),
        AggregatedMetrics::F64(MetricData::Gauge(_))
    ));
    let scope = InstrumentationScope::builder("external")
        .with_version("1.0")
        .with_schema_url("https://example.com/schema")
        .with_attributes(attributes)
        .build();
    let rm = ResourceMetrics::builder()
        .with_resource(Resource::builder_empty().with_service_name("test").build())
        .with_scope_metrics(vec![ScopeMetrics::builder()
            .with_scope(scope.clone())
            .with_metrics(vec![metric])
            .build()])
        .build();
    assert_eq!(
        rm.resource().get(&Key::new("service.name")),
        Some("test".into())
    );
    let scopes: Vec<_> = rm.scope_metrics().collect();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0].scope(), &scope);
    assert_eq!(scopes[0].metrics().count(), 1);
}

#[test]
fn sum_preserves_signed_values_and_temporality() {
    let (start, time) = times();
    let attributes = vec![KeyValue::new("region", "west")];
    let exemplar = Exemplar::builder(-7_i64, time).build();
    let point = SumDataPoint::builder(-7_i64)
        .with_attributes(attributes.clone())
        .with_exemplars(vec![exemplar.clone()])
        .build();
    assert_eq!(point.value(), -7);
    assert_eq!(point.attributes().collect::<Vec<_>>(), vec![&attributes[0]]);
    assert_eq!(point.exemplars().collect::<Vec<_>>(), vec![&exemplar]);
    let sum = Sum::new(vec![point], Temporality::Delta, false, start, time);
    assert_eq!(sum.start_time(), start);
    assert_eq!(sum.time(), time);
    assert_eq!(sum.temporality(), Temporality::Delta);
    assert!(!sum.is_monotonic());
    assert_eq!(sum.data_points().count(), 1);
    let data: AggregatedMetrics = MetricData::from(sum).into();
    assert!(matches!(data, AggregatedMetrics::I64(MetricData::Sum(_))));
}

#[test]
fn histogram_preserves_distribution_and_exemplars() {
    let (start, time) = times();
    let attributes = vec![KeyValue::new("region", "west")];
    let exemplar = Exemplar::builder(7_u64, time).build();
    let point = HistogramDataPoint::builder(3, 12_u64, vec![2.0, 5.0], vec![1, 1, 1])
        .with_attributes(attributes.clone())
        .with_min(1)
        .with_max(7)
        .with_exemplars(vec![exemplar.clone()])
        .build();
    assert_eq!(point.count(), 3);
    assert_eq!(point.sum(), 12);
    assert_eq!(point.min(), Some(1));
    assert_eq!(point.max(), Some(7));
    assert_eq!(point.bounds().collect::<Vec<_>>(), vec![2.0, 5.0]);
    assert_eq!(point.bucket_counts().collect::<Vec<_>>(), vec![1, 1, 1]);
    assert_eq!(point.attributes().collect::<Vec<_>>(), vec![&attributes[0]]);
    assert_eq!(point.exemplars().collect::<Vec<_>>(), vec![&exemplar]);
    let histogram = Histogram::new(vec![point], Temporality::Cumulative, start, time);
    assert_eq!(histogram.start_time(), start);
    assert_eq!(histogram.time(), time);
    assert_eq!(histogram.temporality(), Temporality::Cumulative);
    assert_eq!(histogram.data_points().count(), 1);
    let data: AggregatedMetrics = MetricData::from(histogram).into();
    assert!(matches!(
        data,
        AggregatedMetrics::U64(MetricData::Histogram(_))
    ));
}

#[test]
fn exponential_histogram_preserves_distribution_and_exemplars() {
    let (start, time) = times();
    let attributes = vec![KeyValue::new("region", "west")];
    let exemplar = Exemplar::builder(1.5, time).build();
    let point = ExponentialHistogramDataPoint::builder(
        3,
        0.75,
        0,
        1,
        ExponentialBucket::new(0, vec![1]),
        ExponentialBucket::new(-1, vec![1]),
    )
    .with_attributes(attributes.clone())
    .with_min(-0.75)
    .with_max(1.5)
    .with_zero_threshold(0.01)
    .with_exemplars(vec![exemplar.clone()])
    .build();
    assert_eq!(point.count(), 3);
    assert_eq!(point.sum(), 0.75);
    assert_eq!(point.min(), Some(-0.75));
    assert_eq!(point.max(), Some(1.5));
    assert_eq!(point.scale(), 0);
    assert_eq!(point.zero_count(), 1);
    assert_eq!(point.zero_threshold(), 0.01);
    assert_eq!(point.positive_bucket().offset(), 0);
    assert_eq!(
        point.positive_bucket().counts().collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(point.negative_bucket().offset(), -1);
    assert_eq!(
        point.negative_bucket().counts().collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(point.attributes().collect::<Vec<_>>(), vec![&attributes[0]]);
    assert_eq!(point.exemplars().collect::<Vec<_>>(), vec![&exemplar]);
    let histogram = ExponentialHistogram::new(vec![point], Temporality::Delta, start, time);
    assert_eq!(histogram.start_time(), start);
    assert_eq!(histogram.time(), time);
    assert_eq!(histogram.temporality(), Temporality::Delta);
    assert_eq!(histogram.data_points().count(), 1);
}

#[test]
fn optional_fields_have_explicit_defaults() {
    let (_, time) = times();
    let rm = ResourceMetrics::builder().build();
    assert_eq!(rm.resource().iter().count(), 0);
    assert_eq!(rm.scope_metrics().count(), 0);
    let scope = ScopeMetrics::builder().build();
    assert_eq!(scope.scope(), &InstrumentationScope::default());
    assert_eq!(scope.metrics().count(), 0);
    let gauge_point = GaugeDataPoint::builder(1_u64).build();
    assert_eq!(gauge_point.attributes().count(), 0);
    assert_eq!(gauge_point.exemplars().count(), 0);
    let gauge = Gauge::builder(vec![gauge_point], time).build();
    assert_eq!(gauge.start_time(), None);
    let metric = Metric::builder("minimal", MetricData::from(gauge).into()).build();
    assert_eq!(metric.description(), "");
    assert_eq!(metric.unit(), "");
    let sum_point = SumDataPoint::builder(1_u64).build();
    assert_eq!(sum_point.attributes().count(), 0);
    assert_eq!(sum_point.exemplars().count(), 0);
    let histogram = HistogramDataPoint::builder(0, 0_u64, vec![], vec![0]).build();
    assert_eq!(histogram.min(), None);
    assert_eq!(histogram.max(), None);
    assert_eq!(histogram.attributes().count(), 0);
    assert_eq!(histogram.exemplars().count(), 0);
    let exponential = ExponentialHistogramDataPoint::builder(
        0,
        0_u64,
        0,
        0,
        ExponentialBucket::new(0, vec![]),
        ExponentialBucket::new(0, vec![]),
    )
    .build();
    assert_eq!(exponential.min(), None);
    assert_eq!(exponential.max(), None);
    assert_eq!(exponential.zero_threshold(), 0.0);
    assert_eq!(exponential.attributes().count(), 0);
    assert_eq!(exponential.exemplars().count(), 0);
    let exemplar = Exemplar::builder(1_u64, time).build();
    assert_eq!(exemplar.filtered_attributes().count(), 0);
    assert_eq!(exemplar.span_id(), &[0; 8]);
    assert_eq!(exemplar.trace_id(), &[0; 16]);
}
