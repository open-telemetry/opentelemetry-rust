extern crate protoc_grpcio;

use protobuf_codegen::Customize;
use protoc_grpcio::compile_grpc_protos;

fn main() {
    compile_grpc_protos(
        &[
            "src/proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto",
            "src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto",
            "src/proto/tracez.proto",
        ],
        &["src/proto/opentelemetry-proto/", "src/proto/"],
        "src/proto",
        Some(Customize {
            expose_fields: Some(true),
            serde_derive: Some(true),
            gen_mod_rs: Some(true),
            ..Default::default()
        }),
    )
    .expect("Error generating protobuf");
}
