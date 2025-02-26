# Changelog

## vNext

## 0.28.0

Released 2025-Feb-10

- Update `opentelemetry` dependency version to 0.28.
- Bump msrv to 1.75.0.

## 0.27.0

Released 2024-Nov-11

- Update `opentelemetry` dependency version to 0.27

- Bump MSRV to 1.70 [#2179](https://github.com/open-telemetry/opentelemetry-rust/pull/2179)

## v0.26.0
Released 2024-Sep-30

- Update `opentelemetry` dependency version to 0.26

## v0.25.0

- Update `opentelemetry` dependency version to 0.25
- Starting with this version, this crate will align with `opentelemetry` crate
  on major,minor versions.
  
## v0.3.0
- Update `opentelemetry` dependency version to 0.24

## v0.2.0

### Changed

- Propagation error will be reported to global error handler [#1640](https://github.com/open-telemetry/opentelemetry-rust/pull/1640)
- Update `opentelemetry` dependency version to 0.23

## v0.1.0

### Added

- As part of the gradual deprecation of the exporter functionality of the opentelemetry-jaeger crate, move the opentelemetry-jaeger propagator functionality to a new crate named opentelemetry-jaeger-propagator [#1487](https://github.com/open-telemetry/opentelemetry-rust/pull/1487)
