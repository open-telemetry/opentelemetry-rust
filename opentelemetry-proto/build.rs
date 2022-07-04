use std::io::Error;

// Grpc related files used by tonic are generated here. Those files re-generate for each build
// so it's up to date.
//
// Grpc related files used by grpcio are maintained at src/proto/grpcio. tests/grpc_build.rs makes
// sure they are up to date.

fn main() -> Result<(), Error> {
    #[cfg(feature = "gen-tonic")]
    tonic_build::configure()
        .build_server(cfg!(feature = "build-server"))
        .build_client(cfg!(feature = "build-client"))
        .compile(
            &[
                "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace_config.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/logs/v1/logs.proto",
                "src/proto/opentelemetry-proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
            ],
            &["src/proto/opentelemetry-proto"],
        )?;

    Ok(())
}
