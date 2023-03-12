#[cfg(feature = "grpc-tonic")]
// The prost currently will generate a non optional deprecated field for labels.
// We cannot assign value to it otherwise clippy will complain.
// We cannot ignore it as it's not an optional field.
// We can remove this after we removed the labels field from proto.
#[allow(deprecated)]
pub(crate) mod tonic {
    use opentelemetry::metrics::MetricsError;
    use opentelemetry::sdk::export::metrics::{
        aggregation::{
            Count, Histogram as SdkHistogram, LastValue, Max, Min, Sum as SdkSum,
            TemporalitySelector,
        },
        Record,
    };
    use opentelemetry::sdk::metrics::aggregators::{
        HistogramAggregator, LastValueAggregator, SumAggregator,
    };
    use opentelemetry::sdk::InstrumentationLibrary;
    use opentelemetry_proto::tonic::metrics::v1::DataPointFlags;
    use opentelemetry_proto::tonic::FromNumber;
    use opentelemetry_proto::tonic::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::KeyValue,
        metrics::v1::{
            metric::Data, number_data_point, AggregationTemporality, Gauge, Histogram,
            HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
        },
    };

    use crate::to_nanos;
    use crate::transform::{CheckpointedMetrics, ResourceWrapper};
    use std::collections::{BTreeMap, HashMap};

    pub(crate) fn record_to_metric(
        record: &Record,
        temporality_selector: &dyn TemporalitySelector,
    ) -> Result<Metric, MetricsError> {
        let descriptor = record.descriptor();
        let aggregator = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
        let attributes = record
            .attributes()
            .iter()
            .map(|kv| kv.into())
            .collect::<Vec<KeyValue>>();
        let temporality: AggregationTemporality = temporality_selector
            .temporality_for(descriptor, aggregator.aggregation().kind())
            .into();
        let kind = descriptor.number_kind();
        Ok(Metric {
            name: descriptor.name().to_string(),
            description: descriptor.description().cloned().unwrap_or_default(),
            unit: descriptor.unit().unwrap_or("").to_string(),
            data: {
                if let Some(last_value) = aggregator.as_any().downcast_ref::<LastValueAggregator>()
                {
                    Some({
                        let (val, sample_time) = last_value.last_value()?;
                        Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                flags: DataPointFlags::FlagNone as u32,
                                attributes,
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
                                flags: DataPointFlags::FlagNone as u32,
                                attributes,
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
                        let (sum, count, min, max, buckets) = (
                            histogram.sum()?,
                            histogram.count()?,
                            histogram.min()?,
                            histogram.max()?,
                            histogram.histogram()?,
                        );
                        Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                flags: DataPointFlags::FlagNone as u32,
                                attributes,
                                start_time_unix_nano: to_nanos(*record.start_time()),
                                time_unix_nano: to_nanos(*record.end_time()),
                                count,
                                sum: Some(sum.to_f64(kind)),
                                min: Some(min.to_f64(kind)),
                                max: Some(max.to_f64(kind)),
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
                    schema_url: resource
                        .schema_url()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    resource: Some(resource.into()),
                    scope_metrics: metric_map
                        .into_iter()
                        .map(|(instrumentation_library, metrics)| ScopeMetrics {
                            schema_url: instrumentation_library
                                .schema_url
                                .clone()
                                .unwrap_or_default()
                                .to_string(),
                            scope: Some(instrumentation_library.into()),
                            metrics: metrics.into_values().collect::<Vec<Metric>>(), // collect values
                        })
                        .collect::<Vec<ScopeMetrics>>(),
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
    #[cfg(feature = "grpc-tonic")]
    mod tonic {
        use crate::transform::metrics::tonic::merge;
        use crate::transform::{record_to_metric, sink, ResourceWrapper};
        use opentelemetry::attributes::AttributeSet;
        use opentelemetry::metrics::MetricsError;
        use opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector;
        use opentelemetry::sdk::export::metrics::record;
        use opentelemetry::sdk::metrics::aggregators::{
            histogram, last_value, Aggregator, SumAggregator,
        };
        use opentelemetry::sdk::metrics::sdk_api::{
            Descriptor, InstrumentKind, Number, NumberKind,
        };
        use opentelemetry::sdk::{InstrumentationLibrary, Resource};
        use opentelemetry::Context;
        use opentelemetry_proto::tonic::metrics::v1::DataPointFlags;
        use opentelemetry_proto::tonic::{
            common::v1::{any_value, AnyValue, KeyValue},
            metrics::v1::{
                metric::Data, number_data_point, Gauge, Histogram, HistogramDataPoint, Metric,
                NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
            },
            Attributes, FromNumber,
        };
        use std::cmp::Ordering;
        use std::sync::Arc;
        use time::macros::datetime;

        fn key_value(key: &str, value: &str) -> KeyValue {
            KeyValue {
                key: key.to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(value.to_string())),
                }),
            }
        }

        fn i64_to_value(val: i64) -> number_data_point::Value {
            number_data_point::Value::AsInt(val)
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
                flags: DataPointFlags::FlagNone as u32,
                attributes: attributes
                    .into_iter()
                    .map(|(key, value)| key_value(key, value))
                    .collect::<Vec<KeyValue>>(),
                start_time_unix_nano: start_time,
                time_unix_nano: end_time,
                value: Some(number_data_point::Value::from_number(
                    value.into(),
                    &NumberKind::I64,
                )),
                exemplars: vec![],
            }
        }

        type InstrumentationLibraryKv = (&'static str, Option<&'static str>);
        type ResourceKv = Vec<(&'static str, &'static str)>;
        type MetricRaw = (&'static str, Vec<DataPointRaw>);
        type DataPointRaw = (Vec<(&'static str, &'static str)>, u64, u64, i64);

        fn convert_to_resource_metrics(
            data: (ResourceKv, Vec<(InstrumentationLibraryKv, Vec<MetricRaw>)>),
        ) -> opentelemetry_proto::tonic::metrics::v1::ResourceMetrics {
            // convert to proto resource
            let attributes: Attributes = data
                .0
                .into_iter()
                .map(|(k, v)| opentelemetry::KeyValue::new(k.to_string(), v.to_string()))
                .collect::<Vec<opentelemetry::KeyValue>>()
                .into();
            let resource = opentelemetry_proto::tonic::resource::v1::Resource {
                attributes: attributes.0,
                dropped_attributes_count: 0,
            };
            let mut scope_metrics = vec![];
            for ((instrumentation_name, instrumentation_version), metrics) in data.1 {
                scope_metrics.push(ScopeMetrics {
                    scope: Some(
                        opentelemetry_proto::tonic::common::v1::InstrumentationScope {
                            name: instrumentation_name.to_string(),
                            attributes: Vec::new(),
                            version: instrumentation_version.unwrap_or("").to_string(),
                            dropped_attributes_count: 0,
                        },
                    ),
                    schema_url: "".to_string(),
                    metrics: metrics
                        .into_iter()
                        .map(|(name, data_points)| get_metric_with_name(name, data_points))
                        .collect::<Vec<Metric>>(),
                });
            }
            ResourceMetrics {
                resource: Some(resource),
                schema_url: "".to_string(),
                scope_metrics,
            }
        }

        // Assert two ResourceMetrics are equal. The challenge here is vectors in ResourceMetrics should
        // be compared as unordered list/set. The currently method sort the input stably, and compare the
        // instance one by one.
        //
        // Based on current implementation of sink function. There are two parts where the order is unknown.
        // The first one is instrumentation_library_metrics in ResourceMetrics.
        // The other is metrics in ScopeMetrics.
        //
        // If we changed the sink function to process the input in parallel, we will have to sort other vectors
        // like data points in Metrics.
        fn assert_resource_metrics(mut expect: ResourceMetrics, mut actual: ResourceMetrics) {
            assert_eq!(
                expect
                    .resource
                    .as_mut()
                    .map(|r| r.attributes.sort_by_key(|kv| kv.key.to_string())),
                actual
                    .resource
                    .as_mut()
                    .map(|r| r.attributes.sort_by_key(|kv| kv.key.to_string()))
            );
            assert_eq!(expect.scope_metrics.len(), actual.scope_metrics.len());
            let sort_instrumentation_library =
                |metric: &ScopeMetrics, other_metric: &ScopeMetrics| match (
                    metric.scope.as_ref(),
                    other_metric.scope.as_ref(),
                ) {
                    (Some(library), Some(other_library)) => library
                        .name
                        .cmp(&other_library.name)
                        .then(library.version.cmp(&other_library.version)),
                    _ => Ordering::Equal,
                };
            let sort_metrics = |metric: &Metric, other_metric: &Metric| {
                metric.name.cmp(&other_metric.name).then(
                    metric
                        .description
                        .cmp(&other_metric.description)
                        .then(metric.unit.cmp(&other_metric.unit)),
                )
            };
            expect.scope_metrics.sort_by(sort_instrumentation_library);
            actual.scope_metrics.sort_by(sort_instrumentation_library);

            for (mut expect, mut actual) in expect
                .scope_metrics
                .into_iter()
                .zip(actual.scope_metrics.into_iter())
            {
                assert_eq!(expect.metrics.len(), actual.metrics.len());

                expect.metrics.sort_by(sort_metrics);
                actual.metrics.sort_by(sort_metrics);

                assert_eq!(expect.metrics, actual.metrics)
            }
        }

        #[test]
        fn test_record_to_metric() -> Result<(), MetricsError> {
            let cx = Context::new();
            let attributes = vec![("test1", "value1"), ("test2", "value2")];
            let str_kv_attributes = attributes
                .iter()
                .cloned()
                .map(|(key, value)| key_value(key, value))
                .collect::<Vec<KeyValue>>();
            let attribute_set = AttributeSet::from_attributes(
                attributes
                    .iter()
                    .cloned()
                    .map(|(k, v)| opentelemetry::KeyValue::new(k, v)),
            );
            let start_time = datetime!(2020-12-25 10:10:0 UTC); // unit nano 1608891000000000000
            let end_time = datetime!(2020-12-25 10:10:30 UTC); // unix nano 1608891030000000000

            // Sum
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    InstrumentKind::Counter,
                    NumberKind::I64,
                    None,
                    None,
                );
                let aggregator = SumAggregator::default();
                let val = Number::from(12_i64);
                aggregator.update(&cx, &val, &descriptor)?;
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &cumulative_temporality_selector())?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Sum(Sum {
                        data_points: vec![NumberDataPoint {
                            flags: DataPointFlags::FlagNone as u32,
                            attributes: str_kv_attributes.clone(),
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            value: Some(i64_to_value(12i64)),
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
                    InstrumentKind::GaugeObserver,
                    NumberKind::I64,
                    None,
                    None,
                );
                let aggregator = last_value();
                let val1 = Number::from(12_i64);
                let val2 = Number::from(14_i64);
                aggregator.update(&cx, &val1, &descriptor)?;
                aggregator.update(&cx, &val2, &descriptor)?;
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &cumulative_temporality_selector())?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            flags: DataPointFlags::FlagNone as u32,
                            attributes: str_kv_attributes.clone(),
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: if let Data::Gauge(gauge) = metric.data.clone().unwrap()
                            {
                                // ignore this field as it is the time the value updated.
                                // It changes every time the test runs
                                gauge.data_points[0].time_unix_nano
                            } else {
                                0
                            },
                            value: Some(i64_to_value(14i64)),
                            exemplars: vec![],
                        }],
                    })),
                };

                assert_eq!(expect, metric);
            }

            // Histogram
            {
                let descriptor = Descriptor::new(
                    "test".to_string(),
                    InstrumentKind::Histogram,
                    NumberKind::I64,
                    None,
                    None,
                );
                let bound = [0.1, 0.2, 0.3];
                let aggregator = histogram(&bound);
                let vals = vec![1i64.into(), 2i64.into(), 3i64.into()];
                for val in vals.iter() {
                    aggregator.update(&cx, val, &descriptor)?;
                }
                let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
                let record = record(
                    &descriptor,
                    &attribute_set,
                    Some(&wrapped_aggregator),
                    start_time.into(),
                    end_time.into(),
                );
                let metric = record_to_metric(&record, &cumulative_temporality_selector())?;

                let expect = Metric {
                    name: "test".to_string(),
                    description: "".to_string(),
                    unit: "".to_string(),
                    data: Some(Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            flags: DataPointFlags::FlagNone as u32,
                            attributes: str_kv_attributes,
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            count: 3,
                            sum: Some(6f64),
                            min: Some(-1.0), // TODO: Not sure what's wrong with this.
                            max: Some(3.0),
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
                        InstrumentationLibrary::new(name, version, None),
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
