# Changelog

## vNext

- Initial release. `SpanExporter`, `MetricExporter` and `LogExporter` write traces, metrics and logs as OTLP JSON lines to stdout, to a file, or to any `std::io::Write` implementation, as described by the [OpenTelemetry Protocol File Exporter](https://opentelemetry.io/docs/specs/otel/protocol/file-exporter/) specification. [#2602](https://github.com/open-telemetry/opentelemetry-rust/issues/2602)
