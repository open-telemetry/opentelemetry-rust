use std::fs::File;

use opentelemetry_proto::tonic::metrics::v1::{MetricsData, ResourceMetrics};

pub struct MetricsAsserter {
    results: Vec<ResourceMetrics>,
    expected: Vec<ResourceMetrics>,
}

impl MetricsAsserter {
    pub fn new(results: Vec<ResourceMetrics>, expected: Vec<ResourceMetrics>) -> Self {
        MetricsAsserter { results, expected }
    }

    pub fn assert(self) {
        self.assert_resource_metrics_eq(&self.results, &self.expected);
    }

    fn assert_resource_metrics_eq(
        &self,
        results: &[ResourceMetrics],
        expected: &[ResourceMetrics],
    ) {
        assert_eq!(results.len(), expected.len());
        for i in 0..results.len() {
            let result_resource_metrics = &results[i];
            let expected_resource_metrics = &expected[i];
            assert_eq!(result_resource_metrics, expected_resource_metrics);
        }
    }
}

// read a file contains ResourceMetrics in json format
pub fn read_metrics_from_json(file: File) -> Vec<ResourceMetrics> {
    let reader = std::io::BufReader::new(file);

    let metrics_data: MetricsData =
        serde_json::from_reader(reader).expect("Failed to read json file");
    metrics_data.resource_metrics
}
