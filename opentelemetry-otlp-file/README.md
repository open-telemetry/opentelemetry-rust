# OpenTelemetry Protocol File Exporter

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains [OpenTelemetry](https://opentelemetry.io/) exporters that write traces, metrics and logs as [OTLP JSON](https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding) lines to stdout, to a file, or to any `std::io::Write` implementation. It implements the [OpenTelemetry Protocol File Exporter](https://opentelemetry.io/docs/specs/otel/protocol/file-exporter/) specification.

> [!WARNING]
> The OTLP File Exporter specification has [Development](https://opentelemetry.io/docs/specs/otel/document-status/) status, so the configuration API of this crate may change. The output is the stable OTLP JSON encoding.

## When to use it

Use these exporters where sending OTLP over the network is not possible or not wanted:

- Function-as-a-Service platforms, such as AWS Lambda, which capture stdout as the function's log stream.
- Containers whose stdout is already collected by a log scraping pipeline.
- Hosts that persist telemetry to local files for reliability.

To send OTLP to a collector over HTTP or gRPC, use [`opentelemetry-otlp`](../opentelemetry-otlp) instead. For human-readable output while debugging, use [`opentelemetry-stdout`](../opentelemetry-stdout).

## Output format

Every export is written as one [JSON Lines](https://jsonlines.org) entry: a single line of compact, UTF-8 encoded JSON terminated by `\n`. `SpanExporter` writes one `TracesData` object per batch of spans, `MetricExporter` writes one `MetricsData` object per collection, and `LogExporter` writes one `LogsData` object per batch of log records.

For example, a batch with a single span:

```json
{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}],"droppedAttributesCount":0,"entityRefs":[]},"scopeSpans":[{"scope":{"name":"checkout","version":"","attributes":[],"droppedAttributesCount":0},"spans":[{"traceId":"0a079c94ed47f19726c508c68784b1ca","spanId":"179fe26c0a822afa","traceState":"","parentSpanId":"","flags":257,"name":"GET /cart","kind":2,"startTimeUnixNano":"1790637708114866590","endTimeUnixNano":"1790637708114868607","attributes":[],"droppedAttributesCount":0,"events":[],"droppedEventsCount":0,"links":[],"droppedLinksCount":0,"status":{"message":"","code":0}}],"schemaUrl":""}],"schemaUrl":""}]}
```

Any OTLP JSON consumer can read these lines, for example the OpenTelemetry Collector's [OTLP JSON file receiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/otlpjsonfilereceiver). The receiver reads lines of up to 1 MiB by default and splits or truncates longer ones, so the telemetry in such a line is lost; in Collector contrib 0.162.0 this happens without an error at the default log level. Set its `max_log_size` option above the longest line you expect, or keep lines short with smaller batches, for example with `OTEL_BSP_MAX_EXPORT_BATCH_SIZE` for spans and `OTEL_BLRP_MAX_EXPORT_BATCH_SIZE` for log records. A metrics line contains every metric stream of a collection, so its length depends on the number of streams instead.

## Getting started

```rust
use opentelemetry_otlp_file::{MetricExporter, SpanExporter};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spans are written to stdout, the default output.
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(SpanExporter::default())
        .build();

    // Metrics are appended to a file, which is created if it does not exist.
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(
            MetricExporter::builder()
                .with_file("/var/log/otel/metrics.jsonl")
                .build()?,
        )
        .build();

    // ... run the instrumented application ...

    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
```

Log records are exported the same way with `LogExporter`; connect the `SdkLoggerProvider` to your logging library with an appender such as [`opentelemetry-appender-tracing`](../opentelemetry-appender-tracing). The [basic example](./examples/basic.rs) exports all three signals:

```sh
# Write every signal to stdout.
cargo run --example basic
# Append each signal to its own file in /tmp/otel.
cargo run --example basic -- /tmp/otel
```

## Destinations

| Builder method | Destination |
| --- | --- |
| `with_stdout()` | The standard output of the process. This is the default. |
| `with_file(path)` | The file at `path`, created if it does not exist and always appended to. |
| `with_writer(writer)` | Any `std::io::Write + Send + 'static` value. |

Files are opened when the exporter is built, so `build()` reports a missing directory or a permission problem. Each line is written and flushed while the exporter holds its writer, and also Rust's `stdout` lock when writing there, so it is complete as soon as the export returns and is never interleaved with other exports or with other output written through Rust's `stdout`, such as `println!`. If a write fails partway through a line, the export returns an error and the rest of the line is written before the next one, so the following lines stay valid; a line of which nothing was written is dropped. Shutting down an exporter flushes and drops its writer, which closes a file that the exporter opened.

The specification requires a file to contain a single type of telemetry, so give each signal its own file. On stdout, the lines share the stream with anything else the process prints; consumers can tell them apart by their top-level key: `resourceSpans`, `resourceMetrics` or `resourceLogs`.

Exports write with the blocking `std::io::Write` API. The SDK's default batch processors and periodic reader export from their own threads, so a slow destination only delays those threads, whereas `with_simple_exporter` writes on the thread that ends the span or emits the log record, which may be running an async task. The experimental processors and readers that run on an async runtime, enabled by the `experimental_*_with_async_runtime` features of `opentelemetry_sdk`, export on that runtime instead: a slow or blocked write, such as stdout connected to a full pipe, blocks one of its worker threads, and the export timeout cannot take effect until the write returns. Prefer the default processors and reader with these exporters.

## AWS Lambda and other short-lived environments

Lambda sends a function's stdout to CloudWatch Logs, so the exported lines land in the function's log group, from where they can be forwarded, for example with a subscription filter, to anything that reads OTLP JSON.

Batch processors and the periodic reader export from background threads. Lambda freezes the execution environment once the handler has returned and no other event is waiting, and does not notify the function beforehand. While the environment is frozen these threads do not run, so whatever they hold is written after the next invocation thaws it, or is lost if Lambda removes the environment first. Lambda sends `SIGTERM` before removing an environment only when an extension is registered, and then allows 500 ms for the shutdown when all registered extensions are internal. Choose one of these approaches:

- Flush the providers at the end of each invocation. Nothing is left in memory between invocations, at the cost of the time each flush adds to the response. Because the periodic reader exports every metric stream on each flush, the function also writes a metrics line on every invocation, however often it runs.

  ```rust
  tracer_provider.force_flush()?;
  meter_provider.force_flush()?;
  logger_provider.force_flush()?;
  ```

- Leave the batch processors and the periodic reader to export on their own schedule, and flush when the environment shuts down, for example from the hook passed to [`lambda_runtime::spawn_graceful_shutdown_handler`](https://docs.rs/lambda_runtime/latest/lambda_runtime/fn.spawn_graceful_shutdown_handler.html), which registers an extension so that the function receives `SIGTERM`. This writes the fewest lines under steady traffic, but telemetry can wait in memory until the next invocation, and is lost if the environment is reset after a timeout or crash.
- Flush from an [internal extension](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-extensions-api.html) registered for `INVOKE` events once the handler has returned. Lambda sends the response without waiting for extensions, so the flush does not delay it, although its duration is still billed.

`with_simple_exporter` writes each span or log record as soon as it ends, so nothing waits in memory, but every record becomes its own line that repeats the resource, which increases the number of CloudWatch log events and the bytes ingested.

With the `provided.al2023` runtime, every exported line becomes one CloudWatch log event, whether the function uses the Text or the JSON log format. A line longer than about 256 KiB, the CloudWatch Logs event size limit, is split across consecutive log events, and only the last part ends with a newline, so a consumer has to join the parts back together. Keep batches small to avoid this, for example with `OTEL_BSP_MAX_EXPORT_BATCH_SIZE` for spans and `OTEL_BLRP_MAX_EXPORT_BATCH_SIZE` for log records.

## Configuration

The Rust SDK does not select exporters from the `OTEL_TRACES_EXPORTER`, `OTEL_METRICS_EXPORTER` and `OTEL_LOGS_EXPORTER` environment variables, so these exporters are configured in code.

`MetricExporter` reads `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`, which accepts `cumulative` (the default), `delta` or `lowmemory`. `MetricExporterBuilder::with_temporality` takes precedence over it. `OTEL_EXPORTER_OTLP_METRICS_DEFAULT_HISTOGRAM_AGGREGATION` is not supported, because the SDK does not let exporters choose the default aggregation; use a view to select the histogram aggregation instead.

## Feature flags

- `trace`: the span exporter. Enabled by default.
- `metrics`: the metric exporter. Enabled by default.
- `logs`: the log exporter. Enabled by default.
- `internal-logs`: report problems, such as an invalid environment variable, through the OpenTelemetry internal logging macros. Enabled by default.

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported version is 1.75.0. The current OpenTelemetry version is NOT guaranteed to build on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions before it will always be supported. For example, if the current stable compiler version is 1.49, the minimum supported version will not be increased past 1.46, three minor versions prior. Increasing the minimum supported compiler version is not considered a semver breaking change as long as doing so complies with this policy.
