# Metric SDK Advanced Configuration Example

This example shows how to customize the OpenTelemetry Rust Metric SDK. This
shows how to change temporality, how to customize the aggregation using the
concept of "Views", and how to supply a custom hasher for the metrics hot path
via `MeterProviderBuilder::with_hasher` (here using `foldhash`). The examples
write output to stdout, but could be replaced with other exporters.

## Usage

Run the following, and the Metrics will be written out to stdout.

```shell
$ cargo run
```
