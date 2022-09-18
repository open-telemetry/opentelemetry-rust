use protobuf_codegen::Customize;
use protoc_grpcio::compile_grpc_protos;
use std::collections::HashMap;
use std::path::PathBuf;

const GRPCIO_PROTO_DIR: &str = "src/proto/grpcio";
const GRPCIO_PROTO_FILES: &[&str] = &[
    "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
    "src/proto/tracez.proto",
];
const GRPCIO_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto/", "src/proto"];
const GRPCIO_OUT_DIR: &str = "src/proto/grpcio";

const TONIC_PROTO_DIR: &str = "src/proto/tonic";
const TONIC_PROTO_FILES: &[&str] = &[
    "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/logs/v1/logs.proto",
    "src/proto/opentelemetry-proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
];
const TONIC_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto"];

// This test helps to keep files generated and used by grpcio update to date.
// If the test fails, it means the generated files has been changed. Please commit the change
// and rerun test. It should pass at the second time.
#[test]
fn build_grpc() {
    let before_build = build_content_map(GRPCIO_PROTO_DIR);
    compile_grpc_protos(
        GRPCIO_PROTO_FILES,
        GRPCIO_INCLUDES,
        GRPCIO_OUT_DIR,
        Some(Customize {
            expose_fields: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        }),
    )
    .expect("Error generating protobuf");
    let after_build = build_content_map(GRPCIO_PROTO_DIR);
    // we cannot use assert_eq! here because it will print both maps when they don't match, which
    // makes the error message unreadable.
    // If you find the test passed locally but not in CI pipeline. Try update the dependency. It may
    // be a new version of protobuf or other dependencies
    // DO NOT use assert_eq! here as it will print all generated file when proto changes.
    assert!(
        before_build == after_build,
        "generated file has changed, please commit the change file and rerun the test"
    );
}

#[test]
fn build_tonic() {
    let before_build = build_content_map(TONIC_PROTO_DIR);

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(TONIC_PROTO_DIR)
        .compile(TONIC_PROTO_FILES, TONIC_INCLUDES)
        .expect("cannot compile protobuf using tonic");
    let after_build = build_content_map(TONIC_PROTO_DIR);
    // see build_grpc for why we cannot use assert! here.
    assert!(
        before_build == after_build,
        "generated file has changed, please commit the change file and rerun the test"
    );
}

fn build_content_map(path: &'static str) -> HashMap<PathBuf, String> {
    std::fs::read_dir(path)
        .expect("cannot open dictionary of generated grpc files")
        .into_iter()
        .flatten()
        .map(|entry| {
            (
                entry.path(),
                std::fs::read_to_string(entry.path()).unwrap_or_else(|_| {
                    panic!("cannot read from file {}", entry.path().to_string_lossy())
                }),
            )
        })
        .collect()
}
