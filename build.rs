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
    protoc_grpcio::compile_grpc_protos(
        &[
            "google/devtools/cloudtrace/v2/tracing.proto",
            "google/devtools/cloudtrace/v2/trace.proto",
            "google/rpc/status.proto",
        ],
        &["proto/googleapis/"],
        &out_dir,
        None,
    )
    .expect("protoc-rust-grpc");
    std::fs::File::create(out_dir + "/mod.rs")
        .unwrap()
        .write_all(MOD_RS)
        .unwrap();
}
