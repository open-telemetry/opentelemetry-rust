# Metric SDK Advanced Configuration Example

This example shows how to customize the OpenTelemetry Rust Metric SDK. This
shows how to change temporality, how to customize the aggregation using the
concept of "Views" etc. The examples write output to stdout, but could be
replaced with other exporters.

## Usage

Run the following, and the Metrics will be written out to stdout.

```shell
$ cargo run --bin metrics-advanced
```

## External metric producer

Run `cargo run -p metrics-advanced --bin metric-producer` from the workspace root.
It constructs pre-aggregated metrics using public builders and exports
`external.requests = 42`, `external.temperature = 23.5`, and `sdk.requests = 7`
to stdout under the SDK resource `service.name = metric-producer-example`.

The producer is registered with `PeriodicReader::builder(exporter).with_producer(...)`.
It returns `ScopeMetrics`; the reader supplies and retains the SDK resource.
Replace the stdout exporter with an OTLP `MetricExporter` to send the same
combined batch to a collector. External data bypasses SDK views and aggregation;
the producer must supply valid points and a temporality supported by the exporter.

If a producer returns an error, the readers omit its data for that collection
but still collect and export SDK metrics and successful producers' metrics.
Producer errors are reported through internal diagnostic warnings when
`internal-logs` is enabled; they do not make collection, flush, or shutdown fail.
SDK collection and export errors still propagate.
