fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/proto",
        includes: &["proto/googleapis/"],
        input: &["proto/googleapis/google/devtools/cloudtrace/v2/trace.proto", "proto/googleapis/google/rpc/status.proto"],
        rust_protobuf: true,
        ..Default::default()
    }).expect("protoc-rust-grpc")
}
