# OpenTelemetry Log Appender for tracing -  Example

This example shows how to use the opentelemetry-appender-tracing crate, which is a
[logging
appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge)
that bridges logs from the [tracing crate](https://tracing.rs/tracing/#events) to
OpenTelemetry. The example setups a LoggerProvider with stdout exporter, so logs
are emitted to stdout.

## Usage

Run the following, and Logs emitted using [tracing](https://docs.rs/tracing/latest/tracing/)
will be written out to stdout.

```shell
cargo run
```
