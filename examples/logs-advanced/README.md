# OpenTelemetry Log Processor Implementation and Composition - Example

This example builds on top of the `logs-basic`, showing how to implement `LogProcessor`s correctly.

The `EnrichmentProcessor` simulates a processor adding information
to the log captured by the OpenTelemetry SDK, which correctly ensures that the
downstream processor's filtering is captured, avoiding unnecessary work.

## Usage

```shell
cargo run --features="spec_unstable_logs_enabled"
```

Notice:
1. only the error log is enriched with the `enriched: true` attribute
2. the enrichment process only happens for the error log, without unnecessary work
