# Changelog

## v0.17.0
## Changed
- Update to `opentelemetry` v0.19.
- Update to `opentelemetry-http` v0.8.
- Update to `opentelemetry-semantic-conventions` v0.11.
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).

## v0.16.0

## Added

- Add support for OTEL_EXPORTER_ZIPKIN_* variables. #718

## Changed

- Add defaults for timeouts to HTTP clients #718
- Update to opentelemetry v0.18.0
- Update to opentelemetry-http v0.7.0
- Update to opentelemetry-semantic-conventions v0.10.0

## v0.15.0

### Changed

- Update to opentelemetry v0.17.0
- Update to opentelemetry-http v0.6.0
- Update to opentelemetry-semantic-conventions v0.9.0

## v0.14.0

### Changed

- Update to opentelemetry v0.16.0

## v0.13.0

### Changed

- Update to opentelemetry v0.15.0

## v0.12.0

### Changed

- Update to opentelemetry v0.14.0
- Remove default surf features #546

## v0.11.0
### Changed
- Update to opentelemetry v0.13.0
- Rename trace config with_default_sampler to with_sampler #482

### Added
- Support for rustls #474

## v0.10.0

### Changed
- Update to opentelemetry v0.12.0
- Update tokio to v1 #421
- Use opentelemetry-http for http integration #415

## v0.9.0

### Changed

- Update to opentelemetry v0.11.0
- Exclude status code if unset #382
- Set `otel.status_code` and `otel.status_description` values #383
- Remove resource reporting #389

## v0.8.0

### Changed

- Update to opentelemetry v0.10.0
- Add MSRV 1.42.0 #296

## v0.7.0

### Changed

- Update typed-builder to 0.7.x #279

## v0.6.0

### Added

- Add `otel.status_code` and `otel.status_message` tags #236
- Export instrument library information #243
- Allow users to choose a custom http client #259

### Changed

- Update to opentelemetry v0.9.0
- Update to use pipeline builder #214

## v0.5.0

### Changed

- Update to opentelemetry v0.8.0

## v0.4.0

### Added
- Added a `with_collector_endpoint` endpoint to config builder #155

### Changed
- Update to opentelemetry v0.7.0

## v0.3.0

### Changed
- Update to opentelemetry v0.6.0

## v0.2.0

### Added
- Support for resource attributes

### Changed
- Update to opentelemetry v0.5.0

### Removed
- `as_any` method on exporter

## v0.1.0

### Changed
- Update to opentelemetry v0.4.0

## v0.0.1

### Added

- Exporter to Zipkin collector through HTTP API

