extern crate protobuf_codegen_pure;

use protobuf_codegen_pure::Codegen;

fn main() {
    let mut codegen = Codegen::default();
    codegen.out_dir("src/proto");
    codegen.inputs(&["src/proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto", "src/proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto"]);
    codegen.includes(&["src/proto/opentelemetry-proto/"]);

    codegen.run().expect("Error generating protobuf");
}
