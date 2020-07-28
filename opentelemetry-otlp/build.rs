extern crate protoc_grpcio;

use protoc_grpcio::compile_grpc_protos;

fn main() {
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
        "src/proto",
        None,
    )
    .expect("Error generating protobuf");
}
