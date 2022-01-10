#[cfg(feature = "tonic")]
// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
pub(crate) mod tonic {
    use opentelemetry_proto::proto::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            metric::Data, number_data_point, AggregationTemporality, Gauge, Histogram,
            HistogramDataPoint, InstrumentationLibraryMetrics, Metric, NumberDataPoint,
            ResourceMetrics, Sum,
        },
    };
    use opentelemetry_proto::transform::metrics::tonic::FromNumber;
    use opentelemetry::metrics::{MetricsError, Number, NumberKind};
    use opentelemetry::sdk::export::metrics::{
        Count, ExportKind, ExportKindFor, Histogram as SdkHistogram, LastValue, Max, Min, Points,
        Record, Sum as SdkSum,
    };
    use opentelemetry::sdk::metrics::aggregators::{
        ArrayAggregator, HistogramAggregator, LastValueAggregator, MinMaxSumCountAggregator,
        SumAggregator,
    };

    use crate::to_nanos;
    use crate::transform::{CheckpointedMetrics, ResourceWrapper};
    use opentelemetry::sdk::InstrumentationLibrary;
    use std::collections::{BTreeMap, HashMap};


    pub(crate) fn record_to_metric(
        record: &Record,
        export_selector: &dyn ExportKindFor,
    ) -> Result<Metric, MetricsError> {
        let descriptor = record.descriptor();
        let aggregator = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
        let attributes = record
            .attributes()
            .iter()
            .map(|kv| kv.into())
            .collect::<Vec<KeyValue>>();
        let temporality: AggregationTemporality =
            export_selector.export_kind_for(descriptor).into();
        let kind = descriptor.number_kind();
        Ok(Metric {
            name: descriptor.name().to_string(),
            description: descriptor
                .description()
                .cloned()
                .unwrap_or_else(|| "".to_string()),
            unit: descriptor.unit().unwrap_or("").to_string(),
            data: {
                if let Some(array) = aggregator.as_any().downcast_ref::<ArrayAggregator>() {
                    if let Ok(points) = array.points() {
                        Some(Data::Gauge(Gauge {
                            data_points: points
                                .into_iter()
                                .map(|val| NumberDataPoint {
                                    attributes: attributes.clone(),
                                    labels: vec![],
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    value: Some(number_data_point::Value::from_number(val, kind)),
                                    exemplars: Vec::default(),
                                })
                                .collect(),
                        }))
                    } else {
                        None
                    }
                } else if let Some(last_value) =
                aggregator.as_any().downcast_ref::<LastValueAggregator>()
                {
                    Some({
                        let (val, sample_time) = last_value.last_value()?;
                        Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes,
                                labels: vec![],
                                start_time_unix_nano: to_nanos(*record.start_time()),
                                time_unix_nano: to_nanos(sample_time),
                                value: Some(number_data_point::Value::from_number(val, kind)),
                                exemplars: Vec::default(),
                            }],
                        })
                    })
                } else if let Some(sum) = aggregator.as_any().downcast_ref::<SumAggregator>() {
                    Some({
                        let val = sum.sum()?;
                        Data::Sum(Sum {
                            data_points: vec![NumberDataPoint {
                                attributes,
                                labels: vec![],
                                start_time_unix_nano: to_nanos(*record.start_time()),
                                time_unix_nano: to_nanos(*record.end_time()),
                                value: Some(number_data_point::Value::from_number(val, kind)),
                                exemplars: Vec::default(),
                            }],
                            aggregation_temporality: temporality as i32,
                            is_monotonic: descriptor.instrument_kind().monotonic(),
                        })
                    })
                } else if let Some(histogram) =
                aggregator.as_any().downcast_ref::<HistogramAggregator>()
                {
                    Some({
                        let (sum, count, buckets) =
                            (histogram.sum()?, histogram.count()?, histogram.histogram()?);
                        Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                attributes,
                                labels: vec![],
                                start_time_unix_nano: to_nanos(*record.start_time()),
                                time_unix_nano: to_nanos(*record.end_time()),
                                count,
                                sum: sum.to_f64(kind),
                                bucket_counts: buckets
                                    .counts()
                                    .iter()
                                    .cloned()
                                    .map(|c| c as u64)
                                    .collect(),
                                explicit_bounds: buckets.boundaries().clone(),
                                exemplars: Vec::default(),
                            }],
                            aggregation_temporality: temporality as i32,
                        })
                    })
                } else if let Some(min_max_sum_count) = aggregator
                    .as_any()
                    .downcast_ref::<MinMaxSumCountAggregator>()
                {
                    Some({
                        let (min, max, sum, count) = (
                            min_max_sum_count.min()?,
                            min_max_sum_count.max()?,
                            min_max_sum_count.sum()?,
                            min_max_sum_count.count()?,
                        );
                        let buckets = vec![min.to_u64(kind), max.to_u64(kind)];
                        let bounds = vec![0.0, 100.0];
                        Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                attributes,
                                labels: vec![],
                                start_time_unix_nano: to_nanos(*record.start_time()),
                                time_unix_nano: to_nanos(*record.end_time()),
                                count,
                                sum: sum.to_f64(kind),
                                bucket_counts: buckets,
                                explicit_bounds: bounds,
                                exemplars: Vec::default(),
                            }],
                            aggregation_temporality: temporality as i32,
                        })
                    })
                } else {
                    None
                }
            },
        })
    }

    // Group metrics with resources and instrumentation libraries with resources first,
    // then instrumentation libraries.
    #[allow(clippy::map_entry)] // caused by https://github.com/rust-lang/rust-clippy/issues/4674
    pub(crate) fn sink(metrics: Vec<CheckpointedMetrics>) -> ExportMetricsServiceRequest {
        let mut sink_map = BTreeMap::<
            ResourceWrapper,
            HashMap<InstrumentationLibrary, HashMap<String, Metric>>,
        >::new();
        for (resource, instrumentation_library, metric) in metrics {
            if sink_map.contains_key(&resource) {
                // found resource, see if we can find instrumentation library
                sink_map.entry(resource).and_modify(|map| {
                    if map.contains_key(&instrumentation_library) {
                        map.entry(instrumentation_library).and_modify(|map| {
                            if map.contains_key(&metric.name) {
                                map.entry(metric.name.clone())
                                    .and_modify(|base| merge(base, metric));
                            } else {
                                map.insert(metric.name.clone(), metric);
                            }
                        });
                    } else {
                        map.insert(instrumentation_library, {
                            let mut map = HashMap::new();
                            map.insert(metric.name.clone(), metric);
                            map
                        });
                    }
                });
            } else {
                // insert resource -> instrumentation library -> metrics
                sink_map.insert(resource, {
                    let mut map = HashMap::new();
                    map.insert(instrumentation_library, {
                        let mut map = HashMap::new();
                        map.insert(metric.name.clone(), metric);
                        map
                    });
                    map
                });
            }
        }

        // convert resource -> instrumentation library -> [metrics] into proto struct ResourceMetric
        ExportMetricsServiceRequest {
            resource_metrics: sink_map
                .into_iter()
                .map(|(resource, metric_map)| ResourceMetrics {
                    resource: Some(resource.into()),
                    instrumentation_library_metrics: metric_map
                        .into_iter()
                        .map(
                            |(instrumentation_library, metrics)| InstrumentationLibraryMetrics {
                                instrumentation_library: Some(instrumentation_library.into()),
                                metrics: metrics
                                    .into_iter()
                                    .map(|(_k, v)| v)
                                    .collect::<Vec<Metric>>(), // collect values
                            },
                        )
                        .collect::<Vec<InstrumentationLibraryMetrics>>(),
                })
                .collect::<Vec<ResourceMetrics>>(),
        }
    }

    // if the data points are the compatible, merge, otherwise do nothing
    macro_rules! merge_compatible_type {
        ($base: ident, $other: ident,
            $ (
                $t:path => $($other_t: path),*
            ) ; *) => {
            match &mut $base.data {
                $(
                    Some($t(base_data)) => {
                        match $other.data {
                            $(
                                Some($other_t(other_data)) => {
                                    if other_data.data_points.len() > 0 {
                                        base_data.data_points.extend(other_data.data_points);
                                    }
                                },
                            )*
                            _ => {}
                        }
                    },
                )*
                _ => {}
            }
        };
    }

    // Merge `other` metric proto struct into base by append its data point.
    // If two metric proto don't have the same type or name, do nothing
    pub(crate) fn merge(base: &mut Metric, other: Metric) {
        if base.name != other.name {
            return;
        }
        merge_compatible_type!(base, other,
            Data::Sum => Data::Sum;
            Data::Gauge => Data::Sum, Data::Gauge;
            Data::Histogram => Data::Histogram;
            Data::Summary => Data::Summary
        );
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    #[cfg(feature = "tonic")]
    mod tonic {
        use opentelemetry_proto::proto::common::v1::{any_value, AnyValue, KeyValue};
        use opentelemetry_proto::proto::metrics::v1::{
            metric::Data, number_data_point, Gauge, Histogram, HistogramDataPoint,
            InstrumentationLibraryMetrics, Metric, NumberDataPoint, ResourceMetrics, Sum,
        };
        use crate::transform::common::tonic::Attributes;
        use crate::transform::metrics::tonic::merge;
        use crate::transform::{record_to_metric, sink, ResourceWrapper};
        use opentelemetry::attributes::AttributeSet;
        use opentelemetry::metrics::{
            Descriptor, InstrumentKind, MetricsError, Number, NumberKind,
        };
        use opentelemetry::sdk::export::metrics::{record, Aggregator, ExportKindSelector};
        use opentelemetry::sdk::metrics::aggregators::{
            histogram, last_value, min_max_sum_count, SumAggregator,
        };
        use opentelemetry::sdk::{InstrumentationLibrary, Resource};
        use std::cmp::Ordering;
        use std::sync::Arc;
        use time::macros::datetime;

        impl From<(&str, &str)> for KeyValue {
            fn from(kv: (&str, &str)) -> Self {
                KeyValue {
                    key: kv.0.to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(kv.1.to_string())),
                    }),
                }
            }
        }

        impl From<i64> for number_data_point::Value {
            fn from(val: i64) -> Self {
                number_data_point::Value::AsInt(val)
            }
        }

        #[allow(clippy::type_complexity)]
        fn get_metric_with_name(
            name: &'static str,
            data_points: Vec<(Vec<(&'static str, &'static str)>, u64, u64, i64)>,
        ) -> Metric {
            Metric {
                name: name.to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::Gauge(Gauge {
                    data_points: data_points
                        .into_iter()
                        .map(|(attributes, start_time, end_time, value)| {
                            get_int_data_point(attributes, start_time, end_time, value)
                        })
                        .collect::<Vec<NumberDataPoint>>(),
                })),
            }
        }

        fn get_int_data_point(
            attributes: Vec<(&'static str, &'static str)>,
            start_time: u64,
            end_time: u64,
            value: i64,
        ) -> NumberDataPoint {
            NumberDataPoint {
                attributes: attributes
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<KeyValue>>(),
                labels: vec![],
                start_time_unix_nano: start_time,
                time_unix_nano: end_time,
                value: Some((Number::from(value), &NumberKind::I64).into()),
                exemplars: vec![],
            }
        }

        type InstrumentationLibraryKv = (&'static str, Option<&'static str>);
        type ResourceKv = Vec<(&'static str, &'static str)>;
        type MetricRaw = (&'static str, Vec<DataPointRaw>);
        type DataPointRaw = (Vec<(&'static str, &'static str)>, u64, u64, i64);

        fn convert_to_resource_metrics(
            data: (ResourceKv, Vec<(InstrumentationLibraryKv, Vec<MetricRaw>)>),
        ) -> opentelemetry_proto::proto::metrics::v1::ResourceMetrics {
            // convert to proto resource
            let attributes: Attributes = data
                .0
                .into_iter()
                .map(|(k, v)| opentelemetry::KeyValue::new(k.to_string(), v.to_string()))
                .collect::<Vec<opentelemetry::KeyValue>>()
                .into();
            let resource = opentelemetry_proto::proto::resource::v1::Resource {
                attributes: attributes.0,
                dropped_attributes_count: 0,
            };
            let mut instrumentation_library_metrics = vec![];
            for ((instrumentation_name, instrumentation_version), metrics) in data.1 {
                instrumentation_library_metrics.push(InstrumentationLibraryMetrics {
                    instrumentation_library: Some(
                        opentelemetry_proto::proto::common::v1::InstrumentationLibrary {
                            name: instrumentation_name.to_string(),
                            version: instrumentation_version.unwrap_or("").to_string(),
                        },
                    ),
                    metrics: metrics
                        .into_iter()
                        .map(|(name, data_points)| get_metric_with_name(name, data_points))
                        .collect::<Vec<Metric>>(),
                });
            }
            ResourceMetrics {
                resource: Some(resource),
                instrumentation_library_metrics,
            }
        }

        // Assert two ResourceMetrics are equal. The challenge here is vectors in ResourceMetrics should
        // be compared as unordered list/set. The currently method sort the input stably, and compare the
        // instance one by one.
        //
        // Based on current implementation of sink function. There are two parts where the order is unknown.
        // The first one is instrumentation_library_metrics in ResourceMetrics.
        // The other is metrics in InstrumentationLibraryMetrics.
        //
        // If we changed the sink function to process the input in parallel, we will have to sort other vectors
        // like data points in Metrics.
        fn assert_resource_metrics(mut expect: ResourceMetrics, mut actual: ResourceMetrics) {
            assert_eq!(expect.resource, actual.resource);
            assert_eq!(
                expect.instrumentation_library_metrics.len(),
                actual.instrumentation_library_metrics.len()
            );
            let sort_instrumentation_library =
                |metric: &InstrumentationLibraryMetrics,
                 other_metric: &InstrumentationLibraryMetrics| {
                    match (
                        metric.instrumentation_library.as_ref(),
                        other_metric.instrumentation_library.as_ref(),
                    ) {
                        (Some(library), Some(other_library)) => library
                            .name
                            .cmp(&other_library.name)
                            .then(library.version.cmp(&other_library.version)),
                        _ => Ordering::Equal,
                    }
                };
            let sort_metrics = |metric: &Metric, other_metric: &Metric| {
                metric.name.cmp(&other_metric.name).then(
                    metric
                        .description
                        .cmp(&other_metric.description)
                        .then(metric.unit.cmp(&other_metric.unit)),
                )
            };
            expect
                .instrumentation_library_metrics
                .sort_by(sort_instrumentation_library);
            actual
                .instrumentation_library_metrics
                .sort_by(sort_instrumentation_library);

            for (mut expect, mut actual) in expect
                .instrumentation_library_metrics
                .into_iter()
                .zip(actual.instrumentation_library_metrics.into_iter())
            {
                assert_eq!(expect.metrics.len(), actual.metrics.len());

                expect.metrics.sort_by(sort_metrics);
                actual.metrics.sort_by(sort_metrics);

                assert_eq!(expect.metrics, actual.metrics)
            }
        }

        #[test]
        fn test_record_to_metric() -> Result<(), MetricsError> {
            let attributes = vec![("test1", "value1"), ("test2", "value2")];
            let str_kv_attributes = attributes
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<KeyValue>>();
            let attribute_set = AttributeSet::from_attributes(
                attributes
                    .iter()
                    .cloned()
                    .map(|(k, v)| opentelemetry::KeyValue::new(k, v)),
            );
            let resource = Resource::new(vec![
                opentelemetry::KeyValue::new("process", "rust"),
                opentelemetry::KeyValue::new("runtime", "sync"),
            ]);
            let start_time = datetime!(2020-12-25 10:10:0 UTC); // unit nano 1608891000000000000
            let end_time = datetime!(2020-12-25 10:10:30 UTC); // unix nano 1608891030000000000

            // Sum
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    "test",
                    None,
                    InstrumentKind::Counter,
                    NumberKind::I64,
                );
                let aggregator = SumAggregator::default();
                let val = Number::from(12_i64);
                aggregator.update(&val, &descriptor)?;
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    &resource,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &ExportKindSelector::Cumulative)?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Sum(Sum {
                        data_points: vec![NumberDataPoint {
                            attributes: str_kv_attributes.clone(),
                            labels: vec![],
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            value: Some(12i64.into()),
                            exemplars: vec![],
                        }],
                        aggregation_temporality: 2,
                        is_monotonic: true,
                    })),
                };

                assert_eq!(expect, metric);
            }

            // Last Value
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    "test",
                    None,
                    InstrumentKind::ValueObserver,
                    NumberKind::I64,
                );
                let aggregator = last_value();
                let val1 = Number::from(12_i64);
                let val2 = Number::from(14_i64);
                aggregator.update(&val1, &descriptor)?;
                aggregator.update(&val2, &descriptor)?;
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    &resource,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &ExportKindSelector::Cumulative)?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            attributes: str_kv_attributes.clone(),
                            labels: vec![],
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: if let Data::Gauge(gauge) = metric.data.clone().unwrap()
                            {
                                // ignore this field as it is the time the value updated.
                                // It changes every time the test runs
                                gauge.data_points[0].time_unix_nano
                            } else {
                                0
                            },
                            value: Some(14i64.into()),
                            exemplars: vec![],
                        }],
                    })),
                };

                assert_eq!(expect, metric);
            }

            // MinMaxSumCount
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    "test",
                    None,
                    InstrumentKind::UpDownSumObserver,
                    NumberKind::I64,
                );
                let aggregator = min_max_sum_count(&descriptor);
                let vals = vec![1i64.into(), 2i64.into(), 3i64.into()];
                for val in vals.iter() {
                    aggregator.update(val, &descriptor)?;
                }
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    &resource,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &ExportKindSelector::Cumulative)?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            attributes: str_kv_attributes.clone(),
                            labels: vec![],
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            count: 3,
                            sum: 6f64,
                            bucket_counts: vec![1, 3],
                            explicit_bounds: vec![0.0, 100.0],
                            exemplars: vec![],
                        }],
                        aggregation_temporality: 2,
                    })),
                };

                assert_eq!(expect, metric);
            }

            // Histogram
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    "test",
                    None,
                    InstrumentKind::ValueRecorder,
                    NumberKind::I64,
                );
                let bound = [0.1, 0.2, 0.3];
                let aggregator = histogram(&descriptor, &bound);
                let vals = vec![1i64.into(), 2i64.into(), 3i64.into()];
                for val in vals.iter() {
                    aggregator.update(val, &descriptor)?;
                }
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    &resource,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &ExportKindSelector::Cumulative)?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            attributes: str_kv_attributes,
                            labels: vec![],
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            count: 3,
                            sum: 6f64,
                            bucket_counts: vec![0, 0, 0, 3],
                            explicit_bounds: vec![0.1, 0.2, 0.3],
                            exemplars: vec![],
                        }],
                        aggregation_temporality: 2,
                    })),
                };

                assert_eq!(expect, metric);
            }

            Ok(())
        }

        #[test]
        fn test_sink() {
            let test_data: Vec<(ResourceWrapper, InstrumentationLibrary, Metric)> = vec![
                (
                    vec![("runtime", "tokio")],
                    ("otlp", Some("0.1.1")),
                    "test",
                    (vec![("attribute1", "attribute2")], 12, 23, 2),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", Some("0.1.1")),
                    "test",
                    (vec![("attribute2", "attribute2")], 16, 19, 20),
                ),
                (
                    vec![("runtime", "tokio"), ("rustc", "v48.0")],
                    ("otlp", Some("0.1.1")),
                    "test",
                    (vec![("attribute2", "attribute2")], 16, 19, 20),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", None),
                    "test",
                    (vec![("attribute1", "attribute2")], 15, 16, 88),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", None),
                    "another_test",
                    (vec![("attribute1", "attribute2")], 15, 16, 99),
                ),
            ]
                .into_iter()
                .map(
                    |(kvs, (name, version), metric_name, (attributes, start_time, end_time, value))| {
                        (
                            ResourceWrapper::from(Resource::new(kvs.into_iter().map(|(k, v)| {
                                opentelemetry::KeyValue::new(k.to_string(), v.to_string())
                            }))),
                            InstrumentationLibrary::new(name, version),
                            get_metric_with_name(
                                metric_name,
                                vec![(attributes, start_time, end_time, value)],
                            ),
                        )
                    },
                )
                .collect::<Vec<(ResourceWrapper, InstrumentationLibrary, Metric)>>();

            let request = sink(test_data);
            let actual = request.resource_metrics;

            let expect = vec![
                (
                    vec![("runtime", "tokio")],
                    vec![
                        (
                            ("otlp", Some("0.1.1")),
                            vec![(
                                "test",
                                vec![
                                    (vec![("attribute1", "attribute2")], 12, 23, 2),
                                    (vec![("attribute2", "attribute2")], 16, 19, 20),
                                ],
                            )],
                        ),
                        (
                            ("otlp", None),
                            vec![
                                (
                                    "test",
                                    vec![(vec![("attribute1", "attribute2")], 15, 16, 88)],
                                ),
                                (
                                    "another_test",
                                    vec![(vec![("attribute1", "attribute2")], 15, 16, 99)],
                                ),
                            ],
                        ),
                    ],
                ),
                (
                    vec![("runtime", "tokio"), ("rustc", "v48.0")],
                    vec![(
                        ("otlp", Some("0.1.1")),
                        vec![(
                            "test",
                            vec![(vec![("attribute2", "attribute2")], 16, 19, 20)],
                        )],
                    )],
                ),
            ]
                .into_iter()
                .map(convert_to_resource_metrics);

            for (expect, actual) in expect.into_iter().zip(actual.into_iter()) {
                assert_resource_metrics(expect, actual);
            }
        }

        #[test]
        fn test_merge() {
            let data_point_base = get_int_data_point(vec![("method", "POST")], 12, 12, 3);
            let data_point_addon = get_int_data_point(vec![("method", "PUT")], 12, 12, 3);

            let mut metric1 = Metric {
                name: "test".to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::Sum(Sum {
                    data_points: vec![data_point_base.clone()],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            let metric2 = Metric {
                name: "test".to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::Sum(Sum {
                    data_points: vec![data_point_addon.clone()],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            let expect = Metric {
                name: "test".to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::Sum(Sum {
                    data_points: vec![data_point_base, data_point_addon],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            merge(&mut metric1, metric2);

            assert_eq!(metric1, expect);
        }
    }
}
