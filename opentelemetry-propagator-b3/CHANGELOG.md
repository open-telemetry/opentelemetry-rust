# Changelog

## vNext

- The B3 propagator's internal "deferred" marker no longer overlaps the W3C
  `random-trace-id` trace flag; span contexts carrying that flag now inject their
  sampling state correctly
  ([#3270](https://github.com/open-telemetry/opentelemetry-rust/issues/3270)).

## 0.33.0

Released 2026-Sep-22

- Add a standalone B3 propagator crate, extracted from `opentelemetry-zipkin` with the existing API and behavior.
