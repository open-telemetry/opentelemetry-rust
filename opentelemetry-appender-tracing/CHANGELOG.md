# Changelog

## vNext

- [1869](https://github.com/open-telemetry/opentelemetry-rust/pull/1869) Utilize the `LogRecord::set_target()` method to pass the tracing target to the SDK.
  Exporters might use the target to override the instrumentation scope, which previously contained "opentelemetry-appender-tracing".

## v0.4.0

- Removed unwanted dependency on opentelemetry-sdk.
- Update `opentelemetry` dependency version to 0.23

## v0.3.0

### Added

- New experimental metadata attributes feature (experimental\_metadata\_attributes) [#1380](https://github.com/open-telemetry/opentelemetry-rust/pull/1380)
  - Experimental new attributes for tracing metadata
  - Fixes the following for events emitted using log crate
    - Normalized metadata fields
    - Remove redundant metadata

## v0.2.0

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)

### Added

- Add log appender versions to loggers (#1182)

## v0.1.0

Initial crate release
