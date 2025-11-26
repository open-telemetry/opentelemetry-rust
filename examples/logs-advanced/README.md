# OpenTelemetry Log Processor Implementation and Composition - Example

This example builds on top of the `logs-basic`, showing how to implement and compose
`LogProcessor`s correctly.

The `FilteringLogProcessor` applies a severity-based filtering. The main purpose is
mostly to serve as a way to show how to compose processors by wrapping them into
each other, in a way akin to the [delegation pattern](https://en.wikipedia.org/wiki/Delegation_pattern).

The `EnrichmentProcessor` simulates a (very!) slow processor adding information
to the log captured by the OpenTelemetry SDK, which correctly ensures that the
downstream processor's filtering is captured, avoiding unnecessary work.

## Usage

```shell
cargo run --features="spec_unstable_logs_enabled"
```

Notice:
1. only the error log is enriched with the `enriched: true` attribute
2. the slow enrichment process only happens for the error log, without unnecessary work

## Extra credit

Tweak the `EnrichmentProcessor` by removing the implementation of `event_enabled`.
The default implementation always accepts the event from the wrapped processor, even
when it's set up to ignore a specific event. As a consequence, the slow enrichment
processor will act on every log record, regardless of whether they are ultimately
ignored. As a consequence, the filtering happening upstream will not be respected,
causing info logs being enriched (with the resulting unnecessary work).
