// Grpc related files used by tonic are generated here. Those files re-generate for each build
// so it's up to date.
//
// Grpc related files used by grpcio are maintained at src/proto/grpcio. tests/grpc_build.rs makes
// sure they are up to date.

fn main() {
    #[cfg(feature = "gen-tonic")]
    {
        use std::path::PathBuf;
        #[cfg(not(feature = "build-server"))]
        let build_server = false;
        #[cfg(feature = "build-server")]
        let build_server = true;

        #[cfg(not(feature = "build-client"))]
        let build_client = false;
        #[cfg(feature = "build-client")]
        let build_client = true;

        let out_dir = PathBuf::from(
            std::env::var("OUT_DIR").expect("OUT_DIR should be set by cargo but can't find"),
        )
        .join("tonic");
        std::fs::create_dir_all(out_dir.clone()).expect("cannot create output dir");
        tonic_build::configure()
                .build_server(build_server)
                .build_client(build_client)
                .format(false)
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
}
