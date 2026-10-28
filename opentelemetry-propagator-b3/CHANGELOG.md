# Changelog

## vNext

- The B3 propagator's internal "deferred" marker no longer overlaps the W3C
  `random-trace-id` trace flag; span contexts carrying that flag now inject their
  sampling state correctly. The debug marker likewise moved to a high bit so
  neither overlaps future W3C trace flags
  ([#3270](https://github.com/open-telemetry/opentelemetry-rust/issues/3270)).
- Fix several deviations from the [B3 specification](https://github.com/openzipkin/b3-propagation)
  and the [OpenTelemetry B3 requirements](https://opentelemetry.io/docs/specs/otel/context/api-propagators/#b3-requirements):
  - **Breaking**: `Propagator::new()`, `Propagator::default()` and
    `B3Encoding::UnSpecified` now inject the single `b3` header and `fields()`
    returns `b3`. Use `Propagator::with_encoding(B3Encoding::MultipleHeader)` to
    keep the previous behavior.
  - `B3Encoding::SingleHeader` no longer also injects the multiple `x-b3-*` headers.
  - Injected sampling state is derived only from the sampled and debug flags; the
    W3C `random-trace-id` flag and other unknown bits are ignored, and debug takes
    precedence over a deferred decision.
  - `X-B3-Flags` values other than `1` are ignored instead of overriding `X-B3-Sampled`.
  - An invalid `X-B3-ParentSpanId` now rejects the extracted context, matching the
    single header behavior.
  - The single header debug flag `d` now also marks the span context as sampled,
    since debug implies an accept decision.

## 0.33.1

Released 2026-Oct-08

- Update OpenTelemetry dependencies to 0.33.1.

## 0.33.0

Released 2026-Sep-22

- Add a standalone B3 propagator crate, extracted from `opentelemetry-zipkin` with the existing API and behavior.
