# Basic OpenTelemetry metrics example with custom error handler:

This example shows how to self-diagnose OpenTelemetry by enabling its internal
logs. OpenTelemetry crates publish internal logs when "internal-logs" feature is
enabled. This feature is enabled by default. Internal logs are published using
`tracing` events, and hence, a `tracing` subscriber must be configured without
which the logs are simply discarded.

## Filtering logs from external dependencies of OTLP Exporter:

The example configures a tracing `filter` to restrict logs from external crates
(`hyper`, `tonic`, and `reqwest` etc.) used by the OTLP Exporter to the `error`
level. This helps prevent an infinite loop of log generation when these crates
emit logs that are picked up by the tracing subscriber. This is only a
workaround until [the root
issue](https://github.com/open-telemetry/opentelemetry-rust/issues/761) is
resolved.

## Filtering logs to be send to OpenTelemetry itself

If you use [OpenTelemetry Tracing
Appender](../../opentelemetry-appender-tracing/README.md) to send `tracing` logs
to OpenTelemetry, then enabling OpenTelemetry internal logs can also cause
infinite, recursive logging. You can filter out all OpenTelemetry internal logs
from being sent to [OpenTelemetry Tracing
Appender](../../opentelemetry-appender-tracing/README.md) using a filter, like
"add_directive("opentelemetry=off".parse().unwrap())" being done for tracing's
`FmtSubscriber`.