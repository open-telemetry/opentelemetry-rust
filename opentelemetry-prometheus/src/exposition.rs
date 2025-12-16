use opentelemetry::{Key, Value};
use std::{collections::BTreeMap, fmt::Write};

/// Encodes metrics in Prometheus exposition format (text-based).
///
/// This is a simple text encoder that converts metrics to the Prometheus
/// exposition format without any external dependencies.
#[derive(Debug)]
pub struct ExpositionEncoder {
    buffer: String,
}

impl ExpositionEncoder {
    /// Creates a new encoder with an empty buffer.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Creates a new encoder with a pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Encodes a HELP line for a metric.
    pub fn encode_help(&mut self, name: &str, description: &str) {
        if !description.is_empty() {
            let _ = writeln!(
                &mut self.buffer,
                "# HELP {} {}",
                name,
                escape_help(description)
            );
        }
    }

    /// Encodes a TYPE line for a metric.
    pub fn encode_type(&mut self, name: &str, metric_type: &str) {
        let _ = writeln!(&mut self.buffer, "# TYPE {} {}", name, metric_type);
    }

    /// Encodes both HELP and TYPE lines for a metric header.
    pub fn encode_metric_header(&mut self, name: &str, description: &str, metric_type: &str) {
        self.encode_help(name, description);
        self.encode_type(name, metric_type);
    }

    /// Encodes a metric sample with labels and value.
    pub fn encode_sample(&mut self, name: &str, labels: &[(String, String)], value: f64) {
        let _ = write!(&mut self.buffer, "{}", name);

        if !labels.is_empty() {
            let _ = write!(&mut self.buffer, "{{");
            for (i, (key, val)) in labels.iter().enumerate() {
                if i > 0 {
                    let _ = write!(&mut self.buffer, ",");
                }
                let _ = write!(&mut self.buffer, "{}=\"{}\"", key, escape_label_value(val));
            }
            let _ = write!(&mut self.buffer, "}}");
        }

        let _ = writeln!(&mut self.buffer, " {}", format_value(value));
    }

    /// Encodes a histogram bucket sample.
    pub fn encode_histogram_bucket(
        &mut self,
        name: &str,
        labels: &[(String, String)],
        upper_bound: f64,
        cumulative_count: u64,
    ) {
        let mut bucket_labels = labels.to_vec();
        bucket_labels.push(("le".to_string(), format_value(upper_bound)));

        self.encode_histogram_metric(name, "bucket", &bucket_labels, cumulative_count as f64);
    }

    /// Encodes a histogram sum.
    pub fn encode_histogram_sum(&mut self, name: &str, labels: &[(String, String)], sum: f64) {
        self.encode_histogram_metric(name, "sum", labels, sum);
    }

    /// Encodes a histogram count.
    pub fn encode_histogram_count(&mut self, name: &str, labels: &[(String, String)], count: u64) {
        self.encode_histogram_metric(name, "count", labels, count as f64);
    }

    /// Helper to encode histogram metrics with a suffix.
    fn encode_histogram_metric(
        &mut self,
        name: &str,
        suffix: &str,
        labels: &[(String, String)],
        value: f64,
    ) {
        self.encode_sample(&format!("{}_{}", name, suffix), labels, value);
    }

    /// Returns the encoded content as a string.
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Returns the encoded content as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.buffer.as_bytes()
    }

    /// Clears the buffer for reuse.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for ExpositionEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Escapes special characters according to Prometheus exposition format.
/// If `include_quote` is true, also escapes double quotes (for label values).
fn escape_string(s: &str, include_quote: bool) -> String {
    s.chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '"' if include_quote => vec!['\\', '"'],
            c => vec![c],
        })
        .collect()
}

/// Escapes special characters in label values according to Prometheus exposition format.
fn escape_label_value(s: &str) -> String {
    escape_string(s, true)
}

/// Escapes special characters in help text.
fn escape_help(s: &str) -> String {
    escape_string(s, false)
}

/// Formats a float value for Prometheus exposition format.
fn format_value(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if v.is_nan() {
        "NaN".to_string()
    } else {
        // Use default float formatting
        v.to_string()
    }
}

/// Converts OpenTelemetry Key-Value pairs to label tuples.
pub(crate) fn otel_kv_to_labels<'a>(
    kvs: impl Iterator<Item = (&'a Key, &'a Value)>,
) -> Vec<(String, String)> {
    // Use BTreeMap to sort keys and collect values
    // First collect with original keys to detect exact duplicates
    let mut seen_exact = std::collections::HashMap::<String, String>::new();
    let mut sanitized_map = BTreeMap::<String, Vec<String>>::new();

    for (key, value) in kvs {
        let key_str = key.as_str();
        let value_str = value.to_string();
        
        // If this exact key was seen before, skip it (keep first value)
        if seen_exact.contains_key(key_str) {
            continue;
        }
        seen_exact.insert(key_str.to_string(), value_str.clone());
        
        // Add to sanitized map (different keys may sanitize to same key)
        let sanitized_key = crate::utils::sanitize_prom_kv(key_str);
        sanitized_map.entry(sanitized_key).or_default().push(value_str);
    }

    sanitized_map.into_iter()
        .map(|(key, mut values)| {
            values.sort_unstable();
            (key, values.join(";"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_label_value() {
        assert_eq!(escape_label_value("simple"), "simple");
        assert_eq!(escape_label_value("with\\slash"), "with\\\\slash");
        assert_eq!(escape_label_value("with\nline"), "with\\nline");
        assert_eq!(escape_label_value("with\"quote"), "with\\\"quote");
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(123.45), "123.45");
        assert_eq!(format_value(f64::INFINITY), "+Inf");
        assert_eq!(format_value(f64::NEG_INFINITY), "-Inf");
        assert_eq!(format_value(f64::NAN), "NaN");
    }

    #[test]
    fn test_encoder_basic() {
        let mut encoder = ExpositionEncoder::new();
        encoder.encode_help("test_metric", "A test metric");
        encoder.encode_type("test_metric", "counter");
        encoder.encode_sample(
            "test_metric",
            &[("label".to_string(), "value".to_string())],
            42.0,
        );

        let result = encoder.finish();
        assert!(result.contains("# HELP test_metric A test metric"));
        assert!(result.contains("# TYPE test_metric counter"));
        assert!(result.contains("test_metric{label=\"value\"} 42"));
    }

    #[test]
    fn test_encoder_histogram() {
        let mut encoder = ExpositionEncoder::new();
        encoder.encode_help("test_histogram", "A test histogram");
        encoder.encode_type("test_histogram", "histogram");

        let labels = vec![("method".to_string(), "GET".to_string())];
        encoder.encode_histogram_bucket("test_histogram", &labels, 0.1, 5);
        encoder.encode_histogram_bucket("test_histogram", &labels, 1.0, 10);
        encoder.encode_histogram_bucket("test_histogram", &labels, f64::INFINITY, 15);
        encoder.encode_histogram_sum("test_histogram", &labels, 12.5);
        encoder.encode_histogram_count("test_histogram", &labels, 15);

        let result = encoder.finish();
        assert!(result.contains("test_histogram_bucket{method=\"GET\",le=\"0.1\"} 5"));
        assert!(result.contains("test_histogram_bucket{method=\"GET\",le=\"1\"} 10"));
        assert!(result.contains("test_histogram_bucket{method=\"GET\",le=\"+Inf\"} 15"));
        assert!(result.contains("test_histogram_sum{method=\"GET\"} 12.5"));
        assert!(result.contains("test_histogram_count{method=\"GET\"} 15"));
    }
}
