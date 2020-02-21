use std::io::Write;

static MOD_RS: &[u8] = b"
/// Generated from protobuf.
pub mod tracing;
/// Generated from protobuf.
pub mod trace;
/// Generated from protobuf.
pub mod status;

";

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: &out_dir,
        includes: &["proto/googleapis/"],
        input: &[
            "proto/googleapis/google/devtools/cloudtrace/v2/tracing.proto",
            "proto/googleapis/google/devtools/cloudtrace/v2/trace.proto",
            "proto/googleapis/google/rpc/status.proto",
        ],
        rust_protobuf: true,
        ..Default::default()
    })
    .expect("protoc-rust-grpc");
    std::fs::File::create(out_dir + "/mod.rs").unwrap().write_all(MOD_RS).unwrap();
}
