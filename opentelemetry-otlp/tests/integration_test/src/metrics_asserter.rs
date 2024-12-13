use std::fs::File;

use opentelemetry_proto::tonic::metrics::v1::{metric, MetricsData, ResourceMetrics, ScopeMetrics};

pub struct MetricsAsserter {
    results: Vec<ResourceMetrics>,
    expected: Vec<ResourceMetrics>,
}

impl MetricsAsserter {
    pub fn new(results: Vec<ResourceMetrics>, expected: Vec<ResourceMetrics>) -> Self {
        MetricsAsserter { results, expected }
    }

    pub fn assert(self) {
        let mut results = self.results.clone();
        let mut expected = self.expected.clone();
        self.assert_resource_metrics_eq(&mut results, &mut expected);
    }

    fn assert_resource_metrics_eq(
        &self,
        results: &mut [ResourceMetrics],
        expected: &mut [ResourceMetrics],
    ) {
        assert_eq!(results.len(), expected.len());
        for i in 0..results.len() {
            let mut result_resource_metrics = &mut results.get_mut(i).unwrap();
            let mut expected_resource_metrics = &mut expected.get_mut(i).unwrap();
            assert_eq(*result_resource_metrics, *expected_resource_metrics);
        }
    }
}

pub fn zero_out_scope_metrics_timestamps(scope_metrics: &mut Vec<ScopeMetrics>) {
    for scope_metric in scope_metrics {
        for metric in &mut scope_metric.metrics {
            match &mut metric.data {
                Some(metric::Data::Gauge(gauge)) => {
                    for data_point in &mut gauge.data_points {
                        data_point.start_time_unix_nano = 0;
                        data_point.time_unix_nano = 0;
                    }
                }
                Some(metric::Data::Sum(sum)) => {
                    for data_point in &mut sum.data_points {
                        data_point.start_time_unix_nano = 0;
                        data_point.time_unix_nano = 0;
                    }
                }
                Some(metric::Data::Histogram(hist)) => {
                    for data_point in &mut hist.data_points {
                        data_point.start_time_unix_nano = 0;
                        data_point.time_unix_nano = 0;
                    }
                }
                Some(metric::Data::ExponentialHistogram(ehist)) => {
                    for data_point in &mut ehist.data_points {
                        data_point.start_time_unix_nano = 0;
                        data_point.time_unix_nano = 0;
                    }
                }
                Some(metric::Data::Summary(summary)) => {
                    for data_point in &mut summary.data_points {
                        data_point.start_time_unix_nano = 0;
                        data_point.time_unix_nano = 0;
                    }
                }
                None => {} // Do nothing for metrics with no data
            }
        }
    }
}

///
/// Compare ResourceMetrics to each other. Because we have timestamps that will be
/// different we need to actively handle this, rather than relying on default
/// comparisons.
///
fn assert_eq(
    result_resource_metrics: &mut ResourceMetrics,
    expected_resource_metrics: &mut ResourceMetrics,
) {
    // No timestamps on the resource itself - compare this verbatim
    assert_eq!(
        result_resource_metrics.resource,
        expected_resource_metrics.resource
    );

    // Compare the metrics themselves
    let mut result_scope_metrics = &mut result_resource_metrics.scope_metrics;
    let mut expected_scope_metrics = &mut expected_resource_metrics.scope_metrics;

    // Zero out all the timestamps so we can usual the default comparisons
    zero_out_scope_metrics_timestamps(&mut result_scope_metrics);
    zero_out_scope_metrics_timestamps(&mut expected_scope_metrics);

    assert_eq!(result_scope_metrics, expected_scope_metrics);
}

// read a file contains ResourceMetrics in json format
pub fn read_metrics_from_json(file: File) -> Vec<ResourceMetrics> {
    let reader = std::io::BufReader::new(file);

    let metrics_data: MetricsData =
        serde_json::from_reader(reader).expect("Failed to read json file");
    metrics_data.resource_metrics
}

pub fn read_metrics_from_json_string(json: &String) -> Vec<ResourceMetrics> {
    let metrics_data: MetricsData = serde_json::from_str(json).expect("Failed to read json file");
    metrics_data.resource_metrics
}
