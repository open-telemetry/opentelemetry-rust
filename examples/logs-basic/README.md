# Log Appender for API -  Example

This example shows how to use the opentelemetry-appender-log crate, which is a
[logging appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) that bridges logs from the [log crate](https://docs.rs/log/latest/log/) to OpenTelemetry.
The example setups a LoggerProvider with stdout exporter, so logs are emitted to stdout.

## Usage

Run the following, and Logs emitted using [log](https://docs.rs/log/latest/log/) will be written out to stdout.

```shell
$ cargo run
```



