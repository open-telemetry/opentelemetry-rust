fn main() {
  tonic_build::configure()
    .build_client(true)
    .build_server(false)
    .format(which::which("rustfmt").is_ok())
    .compile(
      &[
        "google/devtools/cloudtrace/v2/tracing.proto",
        "google/devtools/cloudtrace/v2/trace.proto",
        "google/rpc/status.proto",
      ],
      &["proto/googleapis/"],
    )
    .unwrap();
}
