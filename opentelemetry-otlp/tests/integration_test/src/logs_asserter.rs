use opentelemetry_proto::tonic::logs::v1::{LogsData, ResourceLogs};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;

// Given two ResourceLogs, assert that they are equal except for the timestamps
pub struct LogsAsserter {
    results: Vec<ResourceLogs>,
    expected: Vec<ResourceLogs>,
}

impl LogsAsserter {
    // Create a new LogsAsserter
    pub fn new(results: Vec<ResourceLogs>, expected: Vec<ResourceLogs>) -> Self {
        LogsAsserter { results, expected }
    }

    pub fn assert(self) {
        self.assert_resource_logs_eq(&self.results, &self.expected);
    }

    fn assert_resource_logs_eq(&self, results: &[ResourceLogs], expected: &[ResourceLogs]) {
        let mut results_logs = Vec::new();
        let mut expected_logs = Vec::new();

        assert_eq!(results.len(), expected.len());
        for i in 0..results.len() {
            let result_resource_logs = &results[i];
            let expected_resource_logs = &expected[i];
            assert_eq!(
                result_resource_logs.resource,
                expected_resource_logs.resource
            );
            assert_eq!(
                result_resource_logs.schema_url,
                expected_resource_logs.schema_url
            );

            assert_eq!(
                result_resource_logs.scope_logs.len(),
                expected_resource_logs.scope_logs.len()
            );

            for i in 0..result_resource_logs.scope_logs.len() {
                let result_scope_logs = &result_resource_logs.scope_logs[i];
                let expected_scope_logs = &expected_resource_logs.scope_logs[i];

                results_logs.extend(result_scope_logs.log_records.clone());
                expected_logs.extend(expected_scope_logs.log_records.clone());
            }
        }
        assert_eq!(results_logs, expected_logs);
    }
}

// read a file contains ResourceSpans in json format
pub fn read_logs_from_json(file: File) -> Vec<ResourceLogs> {
    println!("Reading logs from file {:?}", file);
    let reader = std::io::BufReader::new(file);

    let log_data: LogsData = serde_json::from_reader(reader).unwrap();
    log_data.resource_logs
}
