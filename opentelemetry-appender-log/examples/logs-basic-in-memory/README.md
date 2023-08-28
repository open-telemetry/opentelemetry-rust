# Log Appender for API with InMemoryLogsExporter -  Example

This example shows how to use the opentelemetry-appender-log crate, which is a
[logging appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) that bridges logs from the [log crate](https://docs.rs/log/latest/log/) to OpenTelemetry.
The example setups a LoggerProvider with a in-memory exporter, so emitted logs are stored in memory.

## Usage

Run the following, and Logs emitted using [log](https://docs.rs/log/latest/log/) will be written out to InMemoryLogsExporter which then will be pulled and printed out.

```shell
$ cargo run
```



