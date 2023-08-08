use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

const GRPCIO_OUT_DIR: &str = "src/proto/grpcio";
const GRPCIO_PROTO_FILES: &[&str] = &[
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
const GRPCIO_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto/", "src/proto"];

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
];
const TONIC_INCLUDES: &[&str] = &["src/proto/opentelemetry-proto"];

// This test helps to keep files generated and used by grpcio update to date.
// If the test fails, it means the generated files has been changed. Please commit the change
// and rerun test. It should pass at the second time.
#[test]
fn build_grpc() {
    let before_build = build_content_map(GRPCIO_OUT_DIR);

    grpcio_compiler::prost_codegen::compile_protos(
        GRPCIO_PROTO_FILES,
        GRPCIO_INCLUDES,
        GRPCIO_OUT_DIR,
    )
    .expect("cannot compile protobuf using grpcio");

    let after_build = build_content_map(GRPCIO_OUT_DIR);
    ensure_files_are_same(before_build, after_build, GRPCIO_OUT_DIR);
}

#[test]
fn build_tonic() {
    let before_build = build_content_map(TONIC_OUT_DIR);

    let out_dir = TempDir::new().expect("failed to create temp dir to store the generated files");

    // build the generated files into OUT_DIR for now so we don't have to touch the src unless we have to
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .server_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
        .client_mod_attribute(".", "#[cfg(feature = \"gen-tonic\")]")
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
    dbg!(&before_build.keys());
    dbg!(&after_build.keys());
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
