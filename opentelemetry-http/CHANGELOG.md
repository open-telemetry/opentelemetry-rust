# Changelog

## vNext

- Add `reqwest-rustls-webkpi-roots` feature flag to configure `reqwest` to use embedded `webkpi-roots`.

## v0.11.1

- Add feature flag enabling users to configure `reqwest` usage to use rustls via
  `reqwest/rustls-tls` feature flag
  [1638](https://github.com/open-telemetry/opentelemetry-rust/pull/1638).

## v0.11.0

### Changed

- **Breaking** Remove built-in support for surf HTTP client [#1537](https://github.com/open-telemetry/opentelemetry-rust/pull/1537)
- **Breaking** Surface non-2xx status codes as errors; change `ResponseExt` trait to return `HttpError` instead of `TraceError`[#1484](https://github.com/open-telemetry/opentelemetry-rust/pull/1484)

## v0.10.0

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)

## v0.9.0

### Changed

- Update to opentelemetry-api v0.20.0

## v0.8.0

### Changed
- Add response headers in response for `HttpClient` implementations [#918](https://github.com/open-telemetry/opentelemetry-rust/pull/918).
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).

## v0.7.0

### Changed

- Update to opentelemetry v0.18.0
- Export `byte` and `http` types #798
- Implementation of collector http client with pure hyper #853

## v0.6.0

### Changed

- Update to opentelemetry v0.17.0

## v0.5.0

### Changed

- Update to opentelemetry v0.16.0

## v0.4.0

### Changed

- Update to opentelemetry v0.15.0

## v0.3.0

### Changed

- Return response from `HttpClient` #511
- Update to opentelemetry v0.14.0

## v0.2.0

### Changed
- Update to opentelemetry v0.13.0

## v0.1.0

### Added

- Opentelemetry integration with http #415
