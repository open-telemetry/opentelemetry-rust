# Changelog

## vNext

- [2101](https://github.com/open-telemetry/opentelemetry-rust/pull/2101) The `log` events emitted via the `tracing` pipeline using the `log-tracing` crate no longer include the target metadata as attributes. Exporters or backends that rely on this attribute should now access the target directly from the `LogRecord::target` field.


## v0.25.0

- Update `opentelemetry` dependency version to 0.25
- Starting with this version, this crate will align with `opentelemetry` crate
  on major,minor versions.
- Reduce heap allocation by using `&'static str` for `SeverityText`.

## v0.5.0

- [1869](https://github.com/open-telemetry/opentelemetry-rust/pull/1869) Utilize the `LogRecord::set_target()` method to pass the tracing target to the SDK.
  Exporters might use the target to override the instrumentation scope, which previously contained "opentelemetry-appender-tracing".

- **Breaking** [1928](https://github.com/open-telemetry/opentelemetry-rust/pull/1928) Insert tracing event name into LogRecord::event_name instead of attributes.
   - If using a custom exporter, then they must serialize this field directly from LogRecord::event_name instead of iterating over the attributes. OTLP Exporter is modified to handle this.
- Update `opentelemetry` dependency version to 0.24

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
