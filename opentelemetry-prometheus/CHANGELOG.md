# Changelog

## v0.12.0

### Changed
- [Breaking] Add `_total` suffix for all counters [#952](https://github.com/open-telemetry/opentelemetry-rust/pull/952).
- Update to `opentelemetry` v0.19.
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).

## v0.11.0

### Changed

- Update to opentelemetry v0.18.0

### Removed

- BREAKING: `PrometheusExporter::new()` removed. Use `ExporterBuilder`. #727

## v0.10.0

### Added

- Added `prometheus-encoding` feature to export prometheus encoders #652
- Added `with_aggregator_selector` option #667

### Changed

- Update prometheus to 0.13 #644
- Update to opentelemetry v0.17.0

### Fixed

- Enable directly constructing a SpanExporter #655

## v0.9.0

### Added

- Add `from_env` to prometheus exporter builder #605
- Adds `Default` implementation to `ExporterBuilder` based on the otel specification environment variables #242

### Changed

- Update to opentelemetry v0.16.0

### Deprecated

- `PrometheusExporter::new()` is deprecated in favor of using `ExporterBuilder`

## v0.8.0

### Changed

- Update to opentelemetry v0.15.0

## v0.7.0

### Changed

- Update to opentelemetry v0.14.0

## v0.6.0

### Added

- Add sanitization of prometheus label names #462

### Changed

- Update to opentelemetry v0.13.0
- Update prometheus dependency #485

## v0.5.0

### Added

- Batch observer support #429

### Changed

- Update to opentelemetry v0.12.0
- Update tokio to v1 #421
- Update prometheus to v0.11 #435

## v0.4.0

### Changed

- Update to opentelemetry v0.11.0
- Add non monotonic counter support #385

## v0.3.0

### Changed

- Update to opentelemetry v0.10.0

## v0.2.0

### Changed

- Update to prometheus 0.10.x #279

## v0.1.0

### Added

- Initial prometheus exporter
