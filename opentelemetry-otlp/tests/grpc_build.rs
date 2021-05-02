use protobuf_codegen::Customize;
use protoc_grpcio::compile_grpc_protos;
use std::collections::HashMap;
use std::path::PathBuf;

// This test helps to keep files generated and used by grpcio update to date.
// If the test fails, it means the generated files has been changed. Please commit the change
// and rerun test. It should pass at the second time.
#[test]
fn build_grpc() {
    let before_build = build_content_map();
    compile_grpc_protos(
        &[
            "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
        ],
        &["src/proto/opentelemetry-proto/"],
        "src/proto/grpcio",
        Some(Customize {
            expose_fields: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        }),
    )
        .expect("Error generating protobuf");
    let after_build = build_content_map();
    // we cannot use assert_eq! here because it will print both maps when they don't match, which
    // makes the error message unreadable.
    // If you find the test passed locally but not in CI pipeline. Try update the dependency. It may
    // be a new version of protobuf or other dependencies
    assert!(
        before_build == after_build,
        "generated file has changed, please commit the change file and rerun the test"
    );
}

fn build_content_map() -> HashMap<PathBuf, String> {
    let mut map = HashMap::new();
    let dict =
        std::fs::read_dir("src/proto/grpcio").expect("cannot open dict of generated grpc files");
    for entry in dict {
        if let Ok(entry) = entry {
            map.insert(
                entry.path(),
                std::fs::read_to_string(entry.path()).unwrap_or_else(|_| {
                    panic!("cannot read from file {}", entry.path().to_string_lossy())
                }),
            );
        }
    }
    map
}
