#[cfg(feature = "tonic")]
pub(crate) mod tonic {
    use crate::proto::{
        collector::metrics::v1::ExportMetricsServiceRequest,
        metrics::v1::{
            metric::Data, AggregationTemporality, DoubleDataPoint, DoubleGauge, DoubleHistogram,
            DoubleHistogramDataPoint, DoubleSum, InstrumentationLibraryMetrics, IntDataPoint,
            IntGauge, IntHistogram, IntHistogramDataPoint, IntSum, Metric, ResourceMetrics,
        },
    };
    use opentelemetry::metrics::{MetricsError, NumberKind};
    use opentelemetry::sdk::export::metrics::{
        Count, ExportKind, ExportKindFor, Histogram, LastValue, Max, Min, Points, Record, Sum,
    };
    use opentelemetry::sdk::metrics::aggregators::{
        ArrayAggregator, HistogramAggregator, LastValueAggregator, MinMaxSumCountAggregator,
        SumAggregator,
    };

    use crate::proto::common::v1::StringKeyValue;
    use crate::transform::common::to_nanos;
    use crate::transform::{CheckpointedMetrics, ResourceWrapper};
    use opentelemetry::sdk::InstrumentationLibrary;
    use opentelemetry::{Key, Value};
    use std::collections::{BTreeMap, HashMap};

    impl From<(&Key, &Value)> for StringKeyValue {
        fn from(kv: (&Key, &Value)) -> Self {
            StringKeyValue {
                key: kv.0.clone().into(),
                value: kv.1.as_str().into(),
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

    pub(crate) fn record_to_metric(
        record: &Record,
        export_selector: &dyn ExportKindFor,
    ) -> Result<Metric, MetricsError> {
        let descriptor = record.descriptor();
        let aggregator = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
        let labels = record
            .labels()
            .iter()
            .map(|kv| kv.into())
            .collect::<Vec<StringKeyValue>>();
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
                        Some({
                            match kind {
                                NumberKind::I64 | NumberKind::U64 => Data::IntGauge(IntGauge {
                                    data_points: points
                                        .into_iter()
                                        .map(|val| IntDataPoint {
                                            labels: labels.clone(),
                                            start_time_unix_nano: to_nanos(*record.start_time()),
                                            time_unix_nano: to_nanos(*record.end_time()),
                                            value: val.to_i64(kind),
                                            exemplars: Vec::default(),
                                        })
                                        .collect(),
                                }),
                                NumberKind::F64 => Data::DoubleGauge(DoubleGauge {
                                    data_points: points
                                        .into_iter()
                                        .map(|val| DoubleDataPoint {
                                            labels: labels.clone(),
                                            start_time_unix_nano: to_nanos(*record.start_time()),
                                            time_unix_nano: to_nanos(*record.end_time()),
                                            value: val.to_f64(kind),
                                            exemplars: Vec::default(),
                                        })
                                        .collect(),
                                }),
                            }
                        })
                    } else {
                        None
                    }
                } else if let Some(last_value) =
                    aggregator.as_any().downcast_ref::<LastValueAggregator>()
                {
                    Some({
                        let (val, sample_time) = last_value.last_value()?;
                        match kind {
                            NumberKind::I64 | NumberKind::U64 => Data::IntGauge(IntGauge {
                                data_points: vec![IntDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(sample_time),
                                    value: val.to_i64(kind),
                                    exemplars: Vec::default(),
                                }],
                            }),
                            NumberKind::F64 => Data::DoubleGauge(DoubleGauge {
                                data_points: vec![DoubleDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(sample_time),
                                    value: val.to_f64(kind),
                                    exemplars: Vec::default(),
                                }],
                            }),
                        }
                    })
                } else if let Some(sum) = aggregator.as_any().downcast_ref::<SumAggregator>() {
                    Some({
                        let val = sum.sum()?;
                        match kind {
                            NumberKind::U64 | NumberKind::I64 => Data::IntSum(IntSum {
                                data_points: vec![IntDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    value: val.to_i64(kind),
                                    exemplars: Vec::default(),
                                }],
                                aggregation_temporality: temporality as i32,
                                is_monotonic: descriptor.instrument_kind().monotonic(),
                            }),
                            NumberKind::F64 => Data::DoubleSum(DoubleSum {
                                data_points: vec![DoubleDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    value: val.to_f64(kind),
                                    exemplars: Vec::default(),
                                }],
                                aggregation_temporality: temporality as i32,
                                is_monotonic: descriptor.instrument_kind().monotonic(),
                            }),
                        }
                    })
                } else if let Some(histogram) =
                    aggregator.as_any().downcast_ref::<HistogramAggregator>()
                {
                    Some({
                        let (sum, count, buckets) =
                            (histogram.sum()?, histogram.count()?, histogram.histogram()?);
                        match kind {
                            NumberKind::I64 | NumberKind::U64 => Data::IntHistogram(IntHistogram {
                                data_points: vec![IntHistogramDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    count,
                                    sum: sum.to_i64(kind),
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
                            }),
                            NumberKind::F64 => Data::DoubleHistogram(DoubleHistogram {
                                data_points: vec![DoubleHistogramDataPoint {
                                    labels,
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
                            }),
                        }
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
                        match kind {
                            NumberKind::U64 | NumberKind::I64 => Data::IntHistogram(IntHistogram {
                                data_points: vec![IntHistogramDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    count,
                                    sum: sum.to_i64(kind),
                                    bucket_counts: buckets,
                                    explicit_bounds: bounds,
                                    exemplars: Vec::default(),
                                }],
                                aggregation_temporality: temporality as i32,
                            }),
                            NumberKind::F64 => Data::DoubleHistogram(DoubleHistogram {
                                data_points: vec![DoubleHistogramDataPoint {
                                    labels,
                                    start_time_unix_nano: to_nanos(*record.start_time()),
                                    time_unix_nano: to_nanos(*record.end_time()),
                                    count,
                                    sum: sum.to_f64(kind),
                                    bucket_counts: buckets,
                                    explicit_bounds: bounds,
                                    exemplars: Vec::default(),
                                }],
                                aggregation_temporality: temporality as i32,
                            }),
                        }
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
            Data::IntSum => Data::IntSum;
            Data::DoubleSum => Data::DoubleSum;
            Data::IntGauge => Data::IntSum, Data::IntGauge;
            Data::DoubleGauge => Data::DoubleSum, Data::DoubleGauge;
            Data::DoubleHistogram => Data::DoubleHistogram;
            Data::IntHistogram => Data::IntHistogram;
            Data::DoubleSummary => Data::DoubleSummary
        );
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "tonic")]
    mod tonic {
        use crate::proto::common::v1::StringKeyValue;
        use crate::proto::metrics::v1::{
            metric::Data, InstrumentationLibraryMetrics, IntDataPoint, IntGauge, IntHistogram,
            IntHistogramDataPoint, IntSum, Metric, ResourceMetrics,
        };
        use crate::transform::common::tonic::Attributes;
        use crate::transform::metrics::tonic::merge;
        use crate::transform::{record_to_metric, sink, ResourceWrapper};
        use chrono::prelude::*;
        use opentelemetry::labels::LabelSet;
        use opentelemetry::metrics::{
            Descriptor, InstrumentKind, MetricsError, Number, NumberKind,
        };
        use opentelemetry::sdk::export::metrics::{record, Aggregator, ExportKindSelector};
        use opentelemetry::sdk::metrics::aggregators::{
            histogram, last_value, min_max_sum_count, SumAggregator,
        };
        use opentelemetry::sdk::{InstrumentationLibrary, Resource};
        use opentelemetry::KeyValue;
        use std::cmp::Ordering;
        use std::sync::Arc;

        #[allow(clippy::type_complexity)]
        fn get_metric_with_name(
            name: &'static str,
            data_points: Vec<(Vec<(&'static str, &'static str)>, u64, u64, i64)>,
        ) -> Metric {
            Metric {
                name: name.to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::IntGauge(IntGauge {
                    data_points: data_points
                        .into_iter()
                        .map(|(labels, start_time, end_time, value)| {
                            get_int_data_point(labels, start_time, end_time, value)
                        })
                        .collect::<Vec<IntDataPoint>>(),
                })),
            }
        }

        fn get_int_data_point(
            labels: Vec<(&'static str, &'static str)>,
            start_time: u64,
            end_time: u64,
            value: i64,
        ) -> IntDataPoint {
            IntDataPoint {
                labels: labels
                    .into_iter()
                    .map(|(k, v)| StringKeyValue {
                        key: k.to_string(),
                        value: v.to_string(),
                    })
                    .collect::<Vec<StringKeyValue>>(),
                start_time_unix_nano: start_time,
                time_unix_nano: end_time,
                value,
                exemplars: vec![],
            }
        }

        type InstrumentationLibraryKv = (&'static str, Option<&'static str>);
        type ResourceKv = Vec<(&'static str, &'static str)>;
        type MetricRaw = (&'static str, Vec<DataPointRaw>);
        type DataPointRaw = (Vec<(&'static str, &'static str)>, u64, u64, i64);

        fn convert_to_resource_metrics(
            data: (ResourceKv, Vec<(InstrumentationLibraryKv, Vec<MetricRaw>)>),
        ) -> crate::proto::metrics::v1::ResourceMetrics {
            // convert to proto resource
            let attributes: Attributes = data
                .0
                .into_iter()
                .map(|(k, v)| KeyValue::new(k.to_string(), v.to_string()))
                .collect::<Vec<KeyValue>>()
                .into();
            let resource = crate::proto::resource::v1::Resource {
                attributes: attributes.0,
                dropped_attributes_count: 0,
            };
            let mut instrumentation_library_metrics = vec![];
            for ((instrumentation_name, instrumentation_version), metrics) in data.1 {
                instrumentation_library_metrics.push(InstrumentationLibraryMetrics {
                    instrumentation_library: Some(
                        crate::proto::common::v1::InstrumentationLibrary {
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
            let labels = vec![("test1", "value1"), ("test2", "value2")];
            let str_kv_labels = labels
                .iter()
                .cloned()
                .map(|(key, value)| StringKeyValue {
                    key: key.to_string(),
                    value: value.to_string(),
                })
                .collect::<Vec<StringKeyValue>>();
            let label_set =
                LabelSet::from_labels(labels.iter().cloned().map(|(k, v)| KeyValue::new(k, v)));
            let resource = Resource::new(vec![
                KeyValue::new("process", "rust"),
                KeyValue::new("runtime", "sync"),
            ]);
            let start_time = Utc.ymd(2020, 12, 25).and_hms(10, 10, 0); // unit nano 1608891000000000000
            let end_time = Utc.ymd(2020, 12, 25).and_hms(10, 10, 30); // unix nano 1608891030000000000

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
                    &label_set,
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
                    data: Some(Data::IntSum(IntSum {
                        data_points: vec![IntDataPoint {
                            labels: str_kv_labels.clone(),
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            value: 12,
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
                    &label_set,
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
                    data: Some(Data::IntGauge(IntGauge {
                        data_points: vec![IntDataPoint {
                            labels: str_kv_labels.clone(),
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: if let Data::IntGauge(gauge) =
                                metric.data.clone().unwrap()
                            {
                                // ignore this field as it is the time the value updated.
                                // It changes every time the test runs
                                gauge.data_points[0].time_unix_nano
                            } else {
                                0
                            },
                            value: 14,
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
                    &label_set,
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
                    data: Some(Data::IntHistogram(IntHistogram {
                        data_points: vec![IntHistogramDataPoint {
                            labels: str_kv_labels.clone(),
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            count: 3,
                            sum: 6,
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
                    &label_set,
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
                    data: Some(Data::IntHistogram(IntHistogram {
                        data_points: vec![IntHistogramDataPoint {
                            labels: str_kv_labels,
                            start_time_unix_nano: 1608891000000000000,
                            time_unix_nano: 1608891030000000000,
                            count: 3,
                            sum: 6,
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
                    (vec![("label1", "label2")], 12, 23, 2),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", Some("0.1.1")),
                    "test",
                    (vec![("label2", "label2")], 16, 19, 20),
                ),
                (
                    vec![("runtime", "tokio"), ("rustc", "v48.0")],
                    ("otlp", Some("0.1.1")),
                    "test",
                    (vec![("label2", "label2")], 16, 19, 20),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", None),
                    "test",
                    (vec![("label1", "label2")], 15, 16, 88),
                ),
                (
                    vec![("runtime", "tokio")],
                    ("otlp", None),
                    "another_test",
                    (vec![("label1", "label2")], 15, 16, 99),
                ),
            ]
            .into_iter()
            .map(
                |(kvs, (name, version), metric_name, (labels, start_time, end_time, value))| {
                    (
                        ResourceWrapper::from(Resource::new(
                            kvs.into_iter()
                                .map(|(k, v)| KeyValue::new(k.to_string(), v.to_string())),
                        )),
                        InstrumentationLibrary::new(name, version),
                        get_metric_with_name(
                            metric_name,
                            vec![(labels, start_time, end_time, value)],
                        ),
                    )
                },
            )
            .collect::<Vec<(ResourceWrapper, InstrumentationLibrary, Metric)>>();

            let request = sink(test_data);
            let actual = request.resource_metrics;

            let expect: Vec<ResourceMetrics> = vec![
                (
                    vec![("runtime", "tokio")],
                    vec![
                        (
                            ("otlp", Some("0.1.1")),
                            vec![(
                                "test",
                                vec![
                                    (vec![("label1", "label2")], 12, 23, 2),
                                    (vec![("label2", "label2")], 16, 19, 20),
                                ],
                            )],
                        ),
                        (
                            ("otlp", None),
                            vec![
                                ("test", vec![(vec![("label1", "label2")], 15, 16, 88)]),
                                (
                                    "another_test",
                                    vec![(vec![("label1", "label2")], 15, 16, 99)],
                                ),
                            ],
                        ),
                    ],
                ),
                (
                    vec![("runtime", "tokio"), ("rustc", "v48.0")],
                    vec![(
                        ("otlp", Some("0.1.1")),
                        vec![("test", vec![(vec![("label2", "label2")], 16, 19, 20)])],
                    )],
                ),
            ]
            .into_iter()
            .map(convert_to_resource_metrics)
            .collect::<Vec<ResourceMetrics>>();

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
                data: Some(Data::IntSum(IntSum {
                    data_points: vec![data_point_base.clone()],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            let metrics2 = Metric {
                name: "test".to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::IntSum(IntSum {
                    data_points: vec![data_point_addon.clone()],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            let expect = Metric {
                name: "test".to_string(),
                description: "".to_string(),
                unit: "".to_string(),
                data: Some(Data::IntSum(IntSum {
                    data_points: vec![data_point_base, data_point_addon],
                    aggregation_temporality: 2,
                    is_monotonic: true,
                })),
            };

            merge(&mut metric1, metrics2);

            assert_eq!(metric1, expect);
        }
    }
}
