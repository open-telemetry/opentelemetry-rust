fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(which::which("rustfmt").is_ok())
        .compile(
            &[
                "google/devtools/cloudtrace/v2/tracing.proto",
                "google/devtools/cloudtrace/v2/trace.proto",
                "google/logging/type/http_request.proto",
                "google/logging/v2/log_entry.proto",
                "google/logging/v2/logging.proto",
                "google/rpc/status.proto",
            ],
            &["proto/googleapis/"],
        )
        .unwrap();
}
