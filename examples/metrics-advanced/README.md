# Metric SDK Advanced Configuration Example

This example shows how to customize the OpenTelemetry Rust Metric SDK. It
demonstrates changing temporality, configuring streams with Views, changing
aggregation and cardinality, and enabling an instrument marked as opt-in. The
examples write output to stdout, but could be replaced with other exporters.

## Usage

Run the following, and the Metrics will be written out to stdout.

```shell
$ cargo run
```

To include the experimental opt-in metric example, run from the repository root:

```shell
cargo run -p metrics-advanced --features experimental_metrics_opt_in
```

The output includes `my_opt_in_counter`, enabled by a matching View. Removing
that View leaves the instrument disabled; recording calls remain unchanged.
This prototype follows [specification PR #4809](https://github.com/open-telemetry/opentelemetry-specification/pull/4809)
and its API may change.
