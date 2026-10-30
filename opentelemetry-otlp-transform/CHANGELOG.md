# Changelog

## vNext

- Initial experimental release with public request conversions from SDK trace,
  metrics, and log data to OTLP protobuf export requests. The public API may
  change before stabilisation. Trace and log callers prepare resource attributes
  once with `ResourceAttributesWithSchema` and reuse them across requests. The
  optional `zpages` feature provides free-function replacements for the former
  `opentelemetry-proto` tracez conversions.
- OTLP span, link, and log record `flags` carry only the W3C-defined trace flags
  (`sampled` and `random-trace-id`); propagator-private bits such as the B3
  propagator's deferred marker are not exported
  ([#3270](https://github.com/open-telemetry/opentelemetry-rust/issues/3270)).
