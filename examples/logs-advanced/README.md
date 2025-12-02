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

## Extra credit

Tweak the `EnrichmentProcessor` by removing the implementation of `event_enabled`.
The default implementation always accepts the event from the wrapped processor, even
when it's set up to ignore a specific event. As a consequence, the enrichment
processor will act on every log record, regardless of whether they are ultimately
ignored. As a consequence, the filtering happening upstream will not be respected,
causing info logs being enriched (with the resulting unnecessary work).
