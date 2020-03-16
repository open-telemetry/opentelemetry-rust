# Changelog

## [v0.2.0](https://github.com/open-telemetry/opentelemetry-rust/compare/b5918251cc07f9f6957434ccddc35306f68bd791..v0.2.0)

### Added
- Make trace and metrics features optional
- ExportResult as specified in the specification
- Add Futures compatibility API
- Added serde serialise support to SpanData
- Separate OpenTelemetry Jaeger crate

### Changed
- Rename HttpTraceContextPropagator to TraceContextPropagator
- Rename HttpB3Propagator to B3Propagator
- Switch to Apache 2 license
- Resolve agent addresses to allow non-static IP
- Remove tracer name prefix from span name

### Removed
- Remove add_link from spans

## [v0.1.5](https://github.com/jtescher/opentelemetry-rust/compare/v0.1.4...v0.1.5)

### Added
- trace-context propagator

### Changed
- Prometheus API cleanup

## [v0.1.4](https://github.com/jtescher/opentelemetry-rust/compare/v0.1.3...v0.1.4)

### Added
- Parent option for default sampler

### Fixed
-  SDK tracer default span id

## [v0.1.3](https://github.com/jtescher/opentelemetry-rust/compare/v0.1.2...v0.1.3)

### Changed
- Ensure spans are always send and sync
- Allow static lifetimes for span names
- Improve KeyValue ergonomics

## [v0.1.2](https://github.com/jtescher/opentelemetry-rust/compare/v0.1.1...v0.1.2)

### Added
- Implement global provider

## [v0.1.1](https://github.com/jtescher/opentelemetry-rust/compare/v0.1.0...v0.1.1)

### Added
- Documentation and API cleanup
- Tracking of active spans via thread local span stack

## [v0.1.0](https://github.com/jtescher/opentelemetry-rust/commit/ea368ea965aa035f46728d75e1be3b096b6cd6ec)

Initial debug alpha
