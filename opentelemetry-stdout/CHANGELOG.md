# Changelog

## vNext

## v0.26.0
Released 2024-Sep-30

- Update `opentelemetry` dependency version to 0.26
- Update `opentelemetry_sdk` dependency version to 0.26

## v0.25.0

- Update `opentelemetry` dependency version to 0.25
- Update `opentelemetry_sdk` dependency version to 0.25
- Starting with this version, this crate will align with `opentelemetry` crate
  on major,minor versions.
- **Breaking** [1994](https://github.com/open-telemetry/opentelemetry-rust/pull/1994) The logrecord event-name is added as attribute with
key `name` only if the feature flag `populate-logs-event-name` is enabled.
- **Breaking** [2040](https://github.com/open-telemetry/opentelemetry-rust/pull/2040) Simplified stdout exporter:
  - Now only supports writing to stdout, removing ability to send telemetry to other streams.
  - Output format improved for better human readability.
  - Note: This exporter is intended for learning and debugging purposes only. Not recommended for production use or automated parsing.

## v0.5.0

- Update `opentelemetry` dependency version to 0.24
- Update `opentelemetry_sdk` dependency version to 0.24

## v0.4.0

### Changed

- The default feature now includes logs, metrics and trace.
- Update `opentelemetry` dependency version to 0.23
- Update `opentelemetry_sdk` dependency version to 0.23
- TraceExporter fixed to print InstrumentationScope's attributes.

## v0.3.0

### Changed

- Fix StatusCode in stdout exporter [#1454](https://github.com/open-telemetry/opentelemetry-rust/pull/1454)
- Add missing event timestamps [#1391](https://github.com/open-telemetry/opentelemetry-rust/pull/1391)
- Adjusted `chrono` features to reduce number of transitive dependencies. [#1569](https://github.com/open-telemetry/opentelemetry-rust/pull/1569)

## v0.2.0

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
- Timestamp is additionally exported in user-friendly format.
  [#1192](https://github.com/open-telemetry/opentelemetry-rust/pull/1192).
- MetricExporter - Temporality is exported in user-friendly format.
  [#1260](https://github.com/open-telemetry/opentelemetry-rust/pull/1260).

## v0.1.0

### Added

- Initial metrics and trace exporters
