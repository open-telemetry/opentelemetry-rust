use anyhow::Result;
use opentelemetry_proto::tonic::logs::v1::{LogRecord, LogsData, ResourceLogs};
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

                assert_eq!(result_scope_logs.scope, expected_scope_logs.scope);

                results_logs.extend(result_scope_logs.log_records.clone());
                expected_logs.extend(expected_scope_logs.log_records.clone());
            }
        }

        for (result_log, expected_log) in results_logs.iter().zip(expected_logs.iter()) {
            assert_eq!(
                LogRecordWrapper(result_log.clone()),
                LogRecordWrapper(expected_log.clone())
            );
        }
    }
}

pub struct LogRecordWrapper(pub LogRecord);

impl PartialEq for LogRecordWrapper {
    fn eq(&self, other: &Self) -> bool {
        let LogRecordWrapper(ref a) = *self;
        let LogRecordWrapper(ref b) = *other;

        assert_eq!(
            a.severity_number, b.severity_number,
            "severity_number does not match"
        );
        assert_eq!(
            a.severity_text, b.severity_text,
            "severity_text does not match"
        );
        assert_eq!(a.body, b.body, "body does not match");
        assert_eq!(a.attributes, b.attributes, "attributes do not match");
        assert_eq!(
            a.dropped_attributes_count, b.dropped_attributes_count,
            "dropped_attributes_count does not match"
        );
        assert_eq!(a.flags, b.flags, "flags do not match");
        assert_eq!(a.trace_id, b.trace_id, "trace_id does not match");
        assert_eq!(a.span_id, b.span_id, "span_id does not match");

        true
    }
}

impl std::fmt::Debug for LogRecordWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LogRecordWrapper(ref inner) = *self;
        inner.fmt(f)
    }
}

// read a file contains ResourceSpans in json format
pub fn read_logs_from_json(file: File) -> Result<Vec<ResourceLogs>> {
    let reader = std::io::BufReader::new(file);

    let log_data: LogsData = serde_json::from_reader(reader)?;
    Ok(log_data.resource_logs)
}
