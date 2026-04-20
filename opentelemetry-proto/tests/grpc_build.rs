use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

const TONIC_OUT_DIR: &str = "src/proto/tonic";
const TONIC_PROTO_FILES: &[&str] = &[
    "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/logs/v1/logs.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/profiles/v1development/profiles.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/profiles/v1development/profiles_service.proto",
    "src/proto/tracez.proto",
];
const TONIC_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto", "src/proto"];

#[test]
fn build_tonic() {
    let before_build = build_content_map(TONIC_OUT_DIR, true);

    let out_dir = TempDir::new().expect("failed to create temp dir to store the generated files");

    // build the generated files into OUT_DIR for now so we don't have to touch the src unless we have to
    let mut builder = tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
        .client_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
        .type_attribute(
            ".",
            "#[cfg_attr(feature = \"with-schemars\", derive(schemars::JsonSchema))]",
        )
        .type_attribute(
            ".",
            "#[cfg_attr(feature = \"with-serde\", derive(serde::Serialize, serde::Deserialize))]",
        )
        .type_attribute(
            ".",
            "#[cfg_attr(feature = \"with-serde\", serde(rename_all = \"camelCase\"))]",
        );

    // Optional numeric, string and array fields need to default to their default value otherwise
    // JSON files without those field cannot deserialize
    // we cannot add serde(default) to all generated types because enums cannot be annotated with serde(default)
    for path in [
        "trace.v1.Span",
        "trace.v1.Span.Link",
        "trace.v1.ScopeSpans",
        "trace.v1.ResourceSpans",
        "common.v1.InstrumentationScope",
        "resource.v1.Resource",
        "trace.v1.Span.Event",
        "trace.v1.Status",
        "logs.v1.LogRecord",
        "logs.v1.ScopeLogs",
        "logs.v1.ResourceLogs",
        "metrics.v1.Metric",
        "metrics.v1.ResourceMetrics",
        "metrics.v1.ScopeMetrics",
        "metrics.v1.Gauge",
        "metrics.v1.Sum",
        "metrics.v1.Histogram",
        "metrics.v1.ExponentialHistogram",
        "metrics.v1.Summary",
        "metrics.v1.NumberDataPoint",
        "metrics.v1.HistogramDataPoint",
        "metrics.v1.SummaryDataPoint",
        "profiles.v1development.Function",
    ] {
        builder = builder.type_attribute(
            path,
            "#[cfg_attr(feature = \"with-serde\", serde(default))]",
        )
    }

    // special serializer and deserializer for traceId and spanId
    // OTLP/JSON format uses hex string for traceId and spanId
    // the proto file uses bytes for traceId and spanId
    // Thus, special serializer and deserializer are needed
    for path in [
        "trace.v1.Span.trace_id",
        "trace.v1.Span.span_id",
        "trace.v1.Span.parent_span_id",
        "trace.v1.Span.Link.trace_id",
        "trace.v1.Span.Link.span_id",
        "logs.v1.LogRecord.span_id",
        "logs.v1.LogRecord.trace_id",
        "metrics.v1.Exemplar.span_id",
        "metrics.v1.Exemplar.trace_id",
        "profiles.v1development.Profile.profile_id",
    ] {
        builder = builder
            .field_attribute(path, "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_to_hex_string\", deserialize_with = \"crate::proto::serializers::deserialize_from_hex_string\"))]")
    }

    // key_strindex is a profiling-only reference field added to common.v1.KeyValue.
    // For non-profiles signals we keep the existing JSON shape by defaulting it on input
    // and omitting it from output when it is zero.
    builder = builder.field_attribute(
        "common.v1.KeyValue.key_strindex",
        "#[cfg_attr(feature = \"with-serde\", serde(default, skip_serializing_if = \"crate::proto::serializers::is_default\"))]",
    );

    // special serializer and deserializer for timestamp
    // OTLP/JSON format may use string for timestamp
    // the proto file uses u64 for timestamp
    // Thus, special serializer and deserializer are needed
    for path in [
        //trace
        "trace.v1.Span.start_time_unix_nano",
        "trace.v1.Span.end_time_unix_nano",
        "trace.v1.Span.Event.time_unix_nano",
        //logs
        "logs.v1.LogRecord.time_unix_nano",
        "logs.v1.LogRecord.observed_time_unix_nano",
        //metrics
        "metrics.v1.HistogramDataPoint.start_time_unix_nano",
        "metrics.v1.HistogramDataPoint.time_unix_nano",
        "metrics.v1.NumberDataPoint.start_time_unix_nano",
        "metrics.v1.NumberDataPoint.time_unix_nano",
        "metrics.v1.ExponentialHistogramDataPoint.start_time_unix_nano",
        "metrics.v1.ExponentialHistogramDataPoint.time_unix_nano",
        "metrics.v1.SummaryDataPoint.start_time_unix_nano",
        "metrics.v1.SummaryDataPoint.time_unix_nano",
        "metrics.v1.Exemplar.time_unix_nano",
    ] {
        builder = builder
            .field_attribute(path, "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_u64_to_string\", deserialize_with = \"crate::proto::serializers::deserialize_string_to_u64\"))]")
    }
    for path in ["profiles.v1development.Profile.time_nanos"] {
        builder = builder
            .field_attribute(path, "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_i64_to_string\", deserialize_with = \"crate::proto::serializers::deserialize_string_to_i64\"))]")
    }
    for path in ["profiles.v1development.Sample.timestamps_unix_nano"] {
        builder = builder
            .field_attribute(path, "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_vec_u64_to_string\", deserialize_with = \"crate::proto::serializers::deserialize_vec_string_to_vec_u64\"))]")
    }

    // special serializer and deserializer for metrics count
    // OTLP/JSON format may use string for count
    // the proto file uses u64 for count
    // Thus, special serializer and deserializer are needed
    for path in [
        // metrics count and bucket fields
        "metrics.v1.HistogramDataPoint.count",
        "metrics.v1.ExponentialHistogramDataPoint.count",
        "metrics.v1.ExponentialHistogramDataPoint.zero_count",
        "metrics.v1.SummaryDataPoint.count",
    ] {
        builder = builder.field_attribute(
            path,
            "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_u64_to_string\", deserialize_with = \"crate::proto::serializers::deserialize_string_to_u64\"))]",
        );
    }

    // special serializer and deserializer for metrics bucket counts
    // OTLP/JSON format may use string for bucket counts
    // the proto file uses u64 for bucket counts
    // Thus, special serializer and deserializer are needed
    for path in [
        "metrics.v1.HistogramDataPoint.bucket_counts",
        "metrics.v1.ExponentialHistogramDataPoint.Buckets.bucket_counts",
    ] {
        builder = builder.field_attribute(
            path,
            "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_vec_u64_to_string\", deserialize_with = \"crate::proto::serializers::deserialize_vec_string_to_vec_u64\"))]",
        );
    }

    // Special handling for floating-point fields that might contain NaN, Infinity, or -Infinity
    // TODO: More needs to be added here as we find more fields that need this special handling
    for path in [
        // metrics
        "metrics.v1.SummaryDataPoint.ValueAtQuantile.value",
        "metrics.v1.SummaryDataPoint.ValueAtQuantile.quantile",
    ] {
        builder = builder.field_attribute(
            path,
            "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_f64_special\", deserialize_with = \"crate::proto::serializers::deserialize_f64_special\"))]",
        );
    }

    // special serializer and deserializer for value
    // The Value::value field must be hidden
    builder = builder
        .field_attribute("common.v1.AnyValue.value", "#[cfg_attr(feature =\"with-serde\", serde(flatten, serialize_with = \"crate::proto::serializers::serialize_to_value\", deserialize_with = \"crate::proto::serializers::deserialize_from_value\"))]");

    // flatten
    for path in ["metrics.v1.Metric.data", "metrics.v1.NumberDataPoint.value"] {
        builder =
            builder.field_attribute(path, "#[cfg_attr(feature =\"with-serde\", serde(flatten))]");
    }

    builder
        .out_dir(out_dir.path())
        .compile_protos(TONIC_PROTO_FILES, TONIC_INCLUDES)
        .expect("cannot compile protobuf using tonic");

    // Post-process each generated file to gate prost-specific derives and types
    // behind the `with-prost` feature, so that http-json users don't pull in prost.
    for entry in std::fs::read_dir(out_dir.path())
        .expect("cannot open temp out dir")
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            let original = std::fs::read_to_string(&path).expect("cannot read generated file");
            let processed = post_process_generated(original);
            std::fs::write(&path, processed).expect("cannot write processed generated file");
        }
    }

    let after_build = build_content_map(out_dir.path(), true);
    ensure_files_are_same(before_build, after_build, TONIC_OUT_DIR);
}

fn build_content_map(path: impl AsRef<Path>, normalize_line_feed: bool) -> HashMap<String, String> {
    std::fs::read_dir(path)
        .expect("cannot open dictionary of generated files")
        .flatten()
        .map(|entry| {
            let path = entry.path();
            let file_name = path
                .file_name()
                .expect("file name should always exist for generated files");

            let mut file_contents = std::fs::read_to_string(path.clone())
                .expect("cannot read from existing generated file");

            if normalize_line_feed {
                file_contents = get_platform_specific_string(file_contents);
            }

            (file_name.to_string_lossy().to_string(), file_contents)
        })
        .collect()
}

///  Returns a String which uses the platform specific new line feed character.
fn get_platform_specific_string(input: String) -> String {
    if cfg!(windows) && !input.ends_with("\r\n") && input.ends_with('\n') {
        return input.replace('\n', "\r\n");
    }
    input
}

fn ensure_files_are_same(
    before_build: HashMap<String, String>,
    after_build: HashMap<String, String>,
    target_dir: &'static str,
) {
    if after_build == before_build {
        return;
    }

    if std::env::var("CI").is_ok() {
        panic!("generated file has changed but it's a CI environment, please rerun this test locally and commit the changes");
    }

    // if there is at least one changes we will just copy the whole directory over
    for (file_name, content) in after_build {
        std::fs::write(Path::new(target_dir).join(file_name), content)
            .expect("cannot write to the proto generate file. If it's happening in CI env, please return the test locally and commit the change");
    }

    panic!("generated file has changed, please commit the change file and rerun the test");
}

/// Post-processes a prost-generated `.rs` file to make all prost-specific
/// derives and types conditional on the `with-prost` feature flag.
///
/// This decouples the struct definitions from `prost` so that consumers using
/// only `http-json` (which needs only `serde` derives) don't transitively
/// compile prost. See: https://github.com/open-telemetry/opentelemetry-rust/issues/3419
fn post_process_generated(content: String) -> String {
    let content = content
        .replace(
            "#[derive(Clone, PartialEq, ::prost::Message)]",
            "#[derive(Clone, PartialEq)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Message))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug, Default))]",
        )
        .replace(
            "#[derive(Clone, Copy, PartialEq, ::prost::Message)]",
            "#[derive(Clone, Copy, PartialEq)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Message))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug, Default))]",
        )
        .replace(
            "#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]",
            "#[derive(Clone, PartialEq, Eq, Hash)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Message))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug, Default))]",
        )
        .replace(
            "#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]",
            "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Message))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug, Default))]",
        )
        .replace(
            "#[derive(Clone, PartialEq, ::prost::Oneof)]",
            "#[derive(Clone, PartialEq)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Oneof))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug))]",
        )
        .replace(
            "#[derive(Clone, Copy, PartialEq, ::prost::Oneof)]",
            "#[derive(Clone, Copy, PartialEq)]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Oneof))]\n#[cfg_attr(not(feature = \"with-prost\"), derive(Debug))]",
        );
    let content = rewrite_prost_enumeration_derives(content);

    let content = content
        .replace("::prost::alloc::vec::Vec", "::std::vec::Vec")
        .replace("::prost::alloc::string::String", "::std::string::String");
    let content = content
        .replace(
            "impl AggregationTemporality {",
            "#[cfg(not(feature = \"with-prost\"))]\nimpl From<AggregationTemporality> for i32 {\n    fn from(value: AggregationTemporality) -> Self {\n        value as i32\n    }\n}\nimpl AggregationTemporality {",
        )
        .replace(
            "impl DataPointFlags {",
            "#[cfg(not(feature = \"with-prost\"))]\nimpl Default for DataPointFlags {\n    fn default() -> Self {\n        Self::DoNotUse\n    }\n}\nimpl DataPointFlags {",
        )
        .replace(
            "impl SeverityNumber {",
            "#[cfg(not(feature = \"with-prost\"))]\nimpl From<SeverityNumber> for i32 {\n    fn from(value: SeverityNumber) -> Self {\n        value as i32\n    }\n}\nimpl SeverityNumber {",
        )
        .replace(
            "impl StatusCode {",
            "#[cfg(not(feature = \"with-prost\"))]\n    impl From<StatusCode> for i32 {\n        fn from(value: StatusCode) -> Self {\n            value as i32\n        }\n    }\n    impl StatusCode {",
        );

    let trailing_newline = content.ends_with('\n');
    let mut out = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("#[prost(") {
                return line.to_string();
            }
            let indent = &line[..line.len() - trimmed.len()];
            // trimmed is "#[prost(ARGS)]"; strip outer #[...] to get "prost(ARGS)"
            let inner = trimmed
                .strip_prefix("#[")
                .and_then(|s| s.strip_suffix(']'))
                .expect("malformed #[prost(...)] attribute in generated file");
            format!("{indent}#[cfg_attr(feature = \"with-prost\", {inner})]")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if trailing_newline {
        out.push('\n');
    }
    out
}

fn rewrite_prost_enumeration_derives(content: String) -> String {
    let mut out = String::with_capacity(content.len());
    let mut rest = content.as_str();

    while let Some(start) = rest.find("#[derive(") {
        let (before, candidate) = rest.split_at(start);
        out.push_str(before);

        let Some(end) = candidate.find(")]") else {
            out.push_str(candidate);
            return out;
        };
        let (block, after) = candidate.split_at(end + 2);

        if !block.contains("::prost::Enumeration") {
            out.push_str(block);
            rest = after;
            continue;
        }

        let inner = block
            .strip_prefix("#[derive(")
            .and_then(|s| s.strip_suffix(")]"))
            .expect("malformed #[derive(...)] attribute in generated file");
        let derives = inner
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty() && *item != "::prost::Enumeration")
            .collect::<Vec<_>>();

        out.push_str(&format!(
            "#[derive({})]\n#[cfg_attr(feature = \"with-prost\", derive(::prost::Enumeration))]",
            derives.join(", ")
        ));
        rest = after;
    }

    out.push_str(rest);
    out
}
