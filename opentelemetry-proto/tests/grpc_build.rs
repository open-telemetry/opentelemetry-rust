use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

const TONIC_OUT_DIR: &str = "src/proto/tonic";
const JSON_OUT_DIR: &str = "src/proto/json";
const PROTO_FILES: &[&str] = &[
    "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/logs/v1/logs.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
];
const TONIC_ONLY_PROTO_FILES: &[&str] = &[
    "src/proto/opentelemetry-proto/opentelemetry/proto/profiles/v1development/profiles.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/profiles/v1development/profiles_service.proto",
    "src/proto/tracez.proto",
];
const INCLUDES: &[&str] = &["src/proto/opentelemetry-proto", "src/proto"];

#[test]
fn build_tonic() {
    let before_build = build_content_map(TONIC_OUT_DIR, true);

    let out_dir = TempDir::new().expect("failed to create temp dir to store the generated files");

    // build the generated files into OUT_DIR for now so we don't have to touch the src unless we have to
    let builder = tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
        .client_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
        .type_attribute(
            ".",
            "#[cfg_attr(feature = \"with-schemars\", derive(schemars::JsonSchema))]",
        );
    configure(builder, "crate::proto::tonic::serializers", |attr| {
        format!("#[cfg_attr(feature = \"with-serde\", {attr})]")
    })
    .out_dir(out_dir.path())
    .compile_protos(&[PROTO_FILES, TONIC_ONLY_PROTO_FILES].concat(), INCLUDES)
    .expect("cannot compile protobuf using tonic");

    let after_build = build_content_map(out_dir.path(), true);
    ensure_files_are_same(before_build, after_build, TONIC_OUT_DIR);
}

#[test]
fn build_json() {
    let before_build = build_content_map(JSON_OUT_DIR, true);

    let out_dir = TempDir::new().expect("failed to create temp dir to store the generated files");

    let builder = tonic_prost_build::configure()
        .build_server(false)
        .build_client(false);
    configure(builder, "crate::proto::json::serializers", |attr| {
        format!("#[{attr}]")
    })
    .out_dir(out_dir.path())
    .compile_protos(PROTO_FILES, INCLUDES)
    .expect("cannot compile protobuf for json");

    for entry in std::fs::read_dir(out_dir.path())
        .expect("cannot open dictionary of generated files")
        .flatten()
    {
        let content = std::fs::read_to_string(entry.path()).expect("cannot read generated file");
        let content = strip_prost(&content);
        assert!(!content.contains("::prost"), "{}", entry.path().display());
        std::fs::write(entry.path(), content).expect("cannot write generated file");
    }

    let after_build = build_content_map(out_dir.path(), true);
    ensure_files_are_same(before_build, after_build, JSON_OUT_DIR);
}

/// The JSON types are the prost-build output with prost removed: `prost::Message`
/// only contributed `Debug` and `Default`, and `prost::alloc` is `std`.
fn strip_prost(generated: &str) -> String {
    generated
        .lines()
        .filter(|line| {
            let item = line.trim_start();
            !item.starts_with("#[prost(") && item != "::prost::Enumeration"
        })
        .flat_map(|line| [line, "\n"])
        .collect::<String>()
        .replace(", ::prost::Message)]", ", Debug, Default)]")
        .replace(", ::prost::Oneof)]", ", Debug)]")
        .replace(", ::prost::Enumeration)]", ")]")
        .replace("::prost::alloc::", "::std::")
}

/// Serde attributes shared by both generated trees. `serde` wraps each attribute body so the
/// tonic tree can gate it on `with-serde`; `serializers` is the module holding the custom
/// (de)serializers of that tree.
fn configure(
    builder: tonic_prost_build::Builder,
    serializers: &str,
    serde: impl Fn(&str) -> String,
) -> tonic_prost_build::Builder {
    let with = |ser: &str, de: &str| {
        serde(&format!(
            "serde(serialize_with = \"{serializers}::{ser}\", deserialize_with = \"{serializers}::{de}\")"
        ))
    };

    let mut builder = builder
        .type_attribute(".", serde("derive(serde::Serialize, serde::Deserialize)"))
        .type_attribute(".", serde("serde(rename_all = \"camelCase\")"));

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
        builder = builder.type_attribute(path, serde("serde(default)"))
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
        builder = builder.field_attribute(
            path,
            with("serialize_to_hex_string", "deserialize_from_hex_string"),
        )
    }

    // key_strindex is a profiling-only reference field added to common.v1.KeyValue.
    // For non-profiles signals we keep the existing JSON shape by defaulting it on input
    // and omitting it from output when it is zero.
    builder = builder.field_attribute(
        "common.v1.KeyValue.key_strindex",
        serde(&format!(
            "serde(default, skip_serializing_if = \"{serializers}::is_default\")"
        )),
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
        // metrics count and bucket fields
        "metrics.v1.HistogramDataPoint.count",
        "metrics.v1.ExponentialHistogramDataPoint.count",
        "metrics.v1.ExponentialHistogramDataPoint.zero_count",
        "metrics.v1.SummaryDataPoint.count",
    ] {
        builder = builder.field_attribute(
            path,
            with("serialize_u64_to_string", "deserialize_string_to_u64"),
        )
    }
    builder = builder.field_attribute(
        "profiles.v1development.Profile.time_nanos",
        with("serialize_i64_to_string", "deserialize_string_to_i64"),
    );
    for path in [
        "profiles.v1development.Sample.timestamps_unix_nano",
        "metrics.v1.HistogramDataPoint.bucket_counts",
        "metrics.v1.ExponentialHistogramDataPoint.Buckets.bucket_counts",
    ] {
        builder = builder.field_attribute(
            path,
            with(
                "serialize_vec_u64_to_string",
                "deserialize_vec_string_to_vec_u64",
            ),
        )
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
            with("serialize_f64_special", "deserialize_f64_special"),
        );
    }

    // special serializer and deserializer for value
    // The Value::value field must be hidden
    builder = builder.field_attribute(
        "common.v1.AnyValue.value",
        serde(&format!(
            "serde(flatten, serialize_with = \"{serializers}::serialize_to_value\", deserialize_with = \"{serializers}::deserialize_from_value\")"
        )),
    );

    // flatten
    for path in ["metrics.v1.Metric.data", "metrics.v1.NumberDataPoint.value"] {
        builder = builder.field_attribute(path, serde("serde(flatten)"));
    }

    builder
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
