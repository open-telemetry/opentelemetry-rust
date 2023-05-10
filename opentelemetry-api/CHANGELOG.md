# Changelog

## Unreleased

### Fixed

- Fix `SpanRef::set_attributes` mutability requirement. [#1038](https://github.com/open-telemetry/opentelemetry-rust/pull/1038)
- Move OrderMap module to root of otel-api crate. [#1061](https://github.com/open-telemetry/opentelemetry-rust/pull/1061)

## v0.19.0
### Added
- Add `WithContext` to public api [#893](https://github.com/open-telemetry/opentelemetry-rust/pull/893).
- Add support for instrumentation scope attributes [#1021](https://github.com/open-telemetry/opentelemetry-rust/pull/1021).

### Changed
- Implement `Display` on `Baggage` [#921](https://github.com/open-telemetry/opentelemetry-rust/pull/921).
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).

## v0.18.0

- API split from `opentelemetry` crate
