#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
extern crate protoc_grpcio;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use protobuf_codegen::Customize;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use protoc_grpcio::compile_grpc_protos;
use std::env;

fn main() {
    #[cfg(feature = "tonic")]
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .format(false)
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

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
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
        env::var("OUT_DIR").unwrap(),
        Some(Customize {
            expose_fields: Some(true),
            serde_derive: Some(true),
            ..Default::default()
        }),
        )
        .expect("Error generating protobuf");
}
