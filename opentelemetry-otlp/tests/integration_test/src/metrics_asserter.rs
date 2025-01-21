use anyhow::Result;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};

pub fn read_metrics_from_json(file: File) -> Result<Value> {
    // Create a buffered reader for the file
    let mut reader = BufReader::new(file);
    let mut contents = String::new();

    // Read the file contents into a string
    reader
        .read_to_string(&mut contents)
        .expect("Failed to read json file");

    // Parse the contents into a JSON Value
    let metrics_data: Value = serde_json::from_str(&contents)?;
    Ok(metrics_data)
}

pub struct MetricsAsserter {
    results: Value,
    expected: Value,
}

impl MetricsAsserter {
    pub fn new(results: Value, expected: Value) -> Self {
        MetricsAsserter { results, expected }
    }

    pub fn assert(mut self) {
        // Normalize JSON by cleaning out timestamps
        Self::zero_out_timestamps(&mut self.results);
        Self::zero_out_timestamps(&mut self.expected);

        // Perform the assertion
        assert_eq!(
            self.results, self.expected,
            "Metrics did not match. Results: {:#?}, Expected: {:#?}",
            self.results, self.expected
        );
    }

    /// Recursively removes or zeros out timestamp fields in the JSON
    fn zero_out_timestamps(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    if key == "startTimeUnixNano" || key == "timeUnixNano" {
                        *val = Value::String("0".to_string());
                    } else {
                        Self::zero_out_timestamps(val);
                    }
                }
            }
            Value::Array(array) => {
                for item in array.iter_mut() {
                    Self::zero_out_timestamps(item);
                }
            }
            _ => {}
        }
    }
}
