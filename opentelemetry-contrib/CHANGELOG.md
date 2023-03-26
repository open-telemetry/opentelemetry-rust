# Changelog

## v0.11.0
### Changed
- Handle `parent_span_id` in jaeger JSON exporter [#907](https://github.com/open-telemetry/opentelemetry-rust/pull/907).
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).
- Implement w3c trace context response propagation [#998](https://github.com/open-telemetry/opentelemetry-rust/pull/998).

## v0.10.0

### Added

- Add jaeger JSON file exporter #814

### Changed

- Rename binary propagator's functions #776
- Update to opentelemetry v0.18.0

## v0.9.0

### Changed

- Update to opentelemetry v0.17.0

## v0.8.0

### Changed

- Update to opentelemetry v0.16.0

## v0.7.0

### Changed

- Update to opentelemetry v0.15.0

## v0.6.0

### Changed

- Update to opentelemetry v0.14.0

## v0.5.0

### Removed
- Moved aws related function to `opentelemetry-aws` crate. #446
- Moved datadog related function to `opentelemetry-datadog` crate. #446

### Changed

- Update to opentelemetry v0.13.0

## v0.4.0

### Changed

- Update to opentelemetry v0.12.0
- Support tokio v1.0 #421
- Use opentelemetry-http for http integration #415

## v0.3.0

### Changed

- Update to opentelemetry v0.11.0

## v0.2.0

### Changed

- Update to opentelemetry v0.10.0
- Move binary propagator and base64 format to this crate #343

## v0.1.0

### Added

- Datadog exporter
