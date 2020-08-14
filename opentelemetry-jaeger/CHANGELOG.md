# Changelog

## v0.7.0

### Changed

- Update to opentelemetry v0.8.0

## v0.6.0

### Changed
- Update to opentelemetry v0.7.0

### Fixed
- Do not add `span.kind` tag if it has been set as an attribute #140

## v0.5.0

### Changed
- Update to opentelemetry v0.6.0

### Fixed
- Switch internally to `ureq` from `reqwest` to fix #106
- Fix exported link span id format #118

## v0.4.0

### Added
- Support for resource attributes

### Changed
- Update to opentelemetry v0.5.0

### Removed
- `as_any` method on exporter

## v0.3.0

### Changed
- Update to opentelemetry v0.4.0

## v0.2.0

### Changed
- Update to opentelemetry v0.3.0

## v0.1.0

### Added
- Jaeger agent Thrift UDP client
- Jaeger collector Thrift HTTP client
