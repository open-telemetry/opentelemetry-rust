use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

use std::fs::File;
use std::io::Write;
use std::process::Command;

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
    "src/proto/tracez.proto",
];
const TONIC_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto", "src/proto"];

#[test]
fn build_tonic() {
    let before_build = build_content_map(TONIC_OUT_DIR);

    let out_dir = TempDir::new().expect("failed to create temp dir to store the generated files");

    // build the generated files into OUT_DIR for now so we don't have to touch the src unless we have to
    let mut builder = tonic_build::configure()
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

    // optional numeric and String field need to default it to 0 otherwise JSON files without those field
    // cannot deserialize
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
    ] {
        builder = builder
            .field_attribute(path, "#[cfg_attr(feature = \"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_to_hex_string\", deserialize_with = \"crate::proto::serializers::deserialize_from_hex_string\"))]")
    }

    // add custom serializer and deserializer for AnyValue
    builder = builder
        .field_attribute("common.v1.KeyValue.value", "#[cfg_attr(feature =\"with-serde\", serde(serialize_with = \"crate::proto::serializers::serialize_to_value\", deserialize_with = \"crate::proto::serializers::deserialize_from_value\"))]");

    builder
        .out_dir(out_dir.path())
        .compile(TONIC_PROTO_FILES, TONIC_INCLUDES)
        .expect("cannot compile protobuf using tonic");

    let after_build = build_content_map(out_dir.path());
    ensure_files_are_same(before_build, after_build, TONIC_OUT_DIR);
}

fn build_content_map(path: impl AsRef<Path>) -> HashMap<String, String> {
    std::fs::read_dir(path)
        .expect("cannot open dictionary of generated files")
        .flatten()
        .map(|entry| {
            let path = entry.path();
            let file_name = path
                .file_name()
                .expect("file name should always exist for generated files");
            (
                file_name.to_string_lossy().to_string(),
                std::fs::read_to_string(path).expect("cannot read from existing generated file"),
            )
        })
        .collect()
}

fn ensure_files_are_same(
    before_build: HashMap<String, String>,
    after_build: HashMap<String, String>,
    target_dir: &'static str,
) {
    let mut changes_detected = false;

    for (file_name, after_content) in &after_build {
        let before_content = before_build.get(file_name);

        if before_content.is_none() || before_content.unwrap() != after_content {
            changes_detected = true;
            println!("File changed: {}", file_name);

            // Temporarily write the after content to a file for diffing
            let temp_after_file_path = format!("/tmp/{}_after", file_name);
            let mut temp_file =
                File::create(&temp_after_file_path).expect("Failed to create temp file");
            writeln!(temp_file, "{}", after_content).expect("Failed to write to temp file");

            let before_content = before_content.cloned().unwrap_or_else(String::new);
            // Temporarily write the before content to a file for diffing
            let temp_before_file_path = format!("/tmp/{}_before", file_name);
            let mut temp_file =
                File::create(&temp_before_file_path).expect("Failed to create temp file");
            writeln!(temp_file, "{}", before_content).expect("Failed to write to temp file");

            // Use the sdiff command to compare the before and after files
            let output = Command::new("sdiff")
                .arg("-s") // Suppress common lines
                .arg(temp_before_file_path)
                .arg(temp_after_file_path)
                .output()
                .expect("Failed to execute sdiff");

            // Print the diff output
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    if changes_detected {
        if std::env::var("CI").is_ok() {
            panic!("Generated files have changed in a CI environment. Please rerun this test locally and commit the changes.");
        } else {
            // If there is at least one change, we will just copy the whole directory over
            for (file_name, content) in after_build {
                std::fs::write(Path::new(target_dir).join(file_name), content)
                    .expect("Cannot write to the proto generate file. If it's happening in CI env, please rerun the test locally and commit the change.");
            }

            panic!(
                "Generated files have changed, please commit the changed files and rerun the test."
            );
        }
    } else {
        println!("No changes detected.");
    }
}
