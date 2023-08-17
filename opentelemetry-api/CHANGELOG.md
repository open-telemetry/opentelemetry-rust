# Changelog

## vNext

### Changed

- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)

## v0.20.0

### Added

- Add `new` method to `BoxedTracer` #1009
- Add js-sys as dependency for api crate when building wasm targets #1078
- Create tracer using a shared instrumentation library #1129
- Add `Context::map_current` #1140
- Add unit/doc tests for metrics #1140

### Changed

- `OtelString::Owned` carries `Box<str>` instead of `String` #1096

### Removed

- Drop include_trace_context parameter from Logs API/SDK. [#1133](https://github.com/open-telemetry/opentelemetry-rust/issues/1133)
- Synchronous instruments no longer accepts `Context` while reporting
  measurements. [#1076](https://github.com/open-telemetry/opentelemetry-rust/pull/1076).

### Fixed

- Fix `SpanRef::set_attributes` mutability requirement. [#1038](https://github.com/open-telemetry/opentelemetry-rust/pull/1038)
- Move OrderMap module to root of otel-api crate. [#1061](https://github.com/open-telemetry/opentelemetry-rust/pull/1061)
- Use the browser-only js-sys workaround only when actually targeting a browser #1008

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
