// Grpc related files used by tonic are generated here. Those files re-generate for each build
// so it's up to date.
//
// Grpc related files used by grpcio are maintained at src/proto/grpcio. tests/grpc_build.rs makes
// sure they are up to date.
#[cfg(any(feature = "tonic", feature = "http-proto"))]
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "tonic")]
    {
        let out_dir = PathBuf::from(
            std::env::var("OUT_DIR").expect("OUT_DIR should be set by cargo but can't find"),
        )
        .join("tonic");
        std::fs::create_dir_all(out_dir.clone()).expect("cannot create output dir");
        tonic_build::configure()
        .build_server(std::env::var_os("CARGO_FEATURE_INTEGRATION_TESTING").is_some())
        .build_client(true)
        .out_dir(out_dir)
        .compile(
            &[
                "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
            ],
            &["src/proto/opentelemetry-proto"],
        )
        .expect("Error generating protobuf");
    }

    #[cfg(feature = "http-proto")]
    {
        let out_dir = PathBuf::from(
            std::env::var("OUT_DIR").expect("OUT_DIR should be set by cargo but can't find"),
        )
        .join("prost");
        std::fs::create_dir_all(out_dir.clone()).expect("cannot create output dir");
        prost_build::Config::new()
            .out_dir(out_dir)
            .compile_protos(
            &[
                "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
            ],
            &["src/proto/opentelemetry-proto"],
            )
        .expect("Error generating protobuf");
    }
}
