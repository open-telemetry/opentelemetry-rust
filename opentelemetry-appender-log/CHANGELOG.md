# Changelog

## vNext

- Bump MSRV to 1.85.0
- "spec_unstable_logs_enabled" feature flag is removed. The capability (and the
  backing specification) is now stable and is enabled by default.
  [3278](https://github.com/open-telemetry/opentelemetry-rust/pull/3278)

## 0.31.0

Released 2025-Sep-25

- Updated `opentelemetry` and `opentelemetry-semantic-conventions` dependencies to version 0.31.0.

## 0.30.0

Released 2025-May-23

- Updated `opentelemetry` and `opentelemetry-semantic-conventions` dependencies to version 0.30.0.

## 0.29.0

Released 2025-Mar-21

- Similar to the `opentelemetry-appender-tracing` fix [2658](https://github.com/open-telemetry/opentelemetry-rust/issues/2658)
  InstrumentationScope(Logger) used by the appender now uses an empty ("") named Logger.
  Previously, a Logger with name and version of the crate was used.
  Receivers (processors, exporters) are expected to use `LogRecord.target()` as scope name.
  This is already done in OTLP Exporters, so this change should be transparent to most users.
- Update `opentelemetry` dependency version to 0.29.
- Update `opentelemetry-semantic-conventions` dependency version to 0.29.

## 0.28.0

Released 2025-Feb-10

- Update `opentelemetry` dependency version to 0.28.
- Update `opentelemetry-semantic-conventions` dependency version to 0.28.
- Bump msrv to 1.75.0.

## 0.27.0

Released 2024-Nov-11

- Update `opentelemetry` dependency version to 0.27

- Bump MSRV to 1.70 [#2179](https://github.com/open-telemetry/opentelemetry-rust/pull/2179)
- [2193](https://github.com/open-telemetry/opentelemetry-rust/pull/2193) `opentelemetry-appender-log`: Output experimental code attributes 
- **Breaking** [2291](https://github.com/open-telemetry/opentelemetry-rust/pull/2291) Rename `logs_level_enabled flag` to `spec_unstable_logs_enabled`. Please enable this updated flag if the feature is needed. This flag will be removed once the feature is stabilized in the specifications.

## v0.26.0
Released 2024-Sep-30
- Update `opentelemetry` dependency version to 0.26

## v0.25.0

- Update `opentelemetry` dependency version to 0.25
- Starting with this version, this crate will align with `opentelemetry` crate
  on major,minor versions.

## v0.5.0

- [1869](https://github.com/open-telemetry/opentelemetry-rust/pull/1869) Utilize the `LogRecord::set_target()` method to pass the log target to the SDK.
- Update `opentelemetry` dependency version to 0.24

## v0.4.0

- Add log key-values as attributes [#1628](https://github.com/open-telemetry/opentelemetry-rust/pull/1628)
- Update `opentelemetry` dependency version to 0.23

## v0.3.0

## v0.2.0

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)

### Fixed

- Add log appender versions to loggers (#1182).

## v0.1.0

Initial crate release
