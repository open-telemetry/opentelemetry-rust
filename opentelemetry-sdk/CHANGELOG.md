# Changelog

## v0.20.0

### Added

- Implement cardinality limits for metric streams
  [#1066](https://github.com/open-telemetry/opentelemetry-rust/pull/1066).
- Propagate shutdown calls from `PeriodicReader` to metrics exporter
  [#1138](https://github.com/open-telemetry/opentelemetry-rust/pull/1138).
- Add in memory metrics exporter #1017

### Changed

- New metrics SDK #1000
- Use `Cow<'static, str>` instead of `&'static str` #1018
- Unify trace and logs runtime extensions traits. #1067

### Changed

- Fix EvictedQueue bug when capacity is set to 0
  [#1151](https://github.com/open-telemetry/opentelemetry-rust/pull/1151).

### Removed

- Samplers no longer has access to `InstrumentationLibrary` as one of parameters
  to `should_sample`.
  [#1041](https://github.com/open-telemetry/opentelemetry-rust/pull/1041).
- Synchronous instruments no longer accepts `Context` while reporting
  measurements. [#1076](https://github.com/open-telemetry/opentelemetry-rust/pull/1076).
- Don't use CARGO_BIN_NAME for service name #1150

### Fixed

- Wait for exports on the simple span processor's ForceFlush #1030

## v0.19.0

### Added
- Add instrument validation to `InstrumentBuilder` [#884](https://github.com/open-telemetry/opentelemetry-rust/pull/884).
- Add `TelemetryResourceDetector` [#899](https://github.com/open-telemetry/opentelemetry-rust/pull/899).
- Add support for instrumentation scope attributes [#1021](https://github.com/open-telemetry/opentelemetry-rust/pull/1021).

### Changed
- Update to `opentelemetry_api` v0.19.
- Update to `opentelemetry_http` v0.8.
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Fix doc in `ShouldSample` trait [#951](https://github.com/open-telemetry/opentelemetry-rust/pull/951)
- Only run `ParentBased` delegate sampler when there is no parent [#948](https://github.com/open-telemetry/opentelemetry-rust/pull/948).
- Improve `SdkProvidedResourceDetector`'s doc [#964](https://github.com/open-telemetry/opentelemetry-rust/pull/964).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).
- Use CARGO_BIN_NAME as default service name [#991](https://github.com/open-telemetry/opentelemetry-rust/pull/991).

### Removed
- Remove `in_memory` settings [#946](https://github.com/open-telemetry/opentelemetry-rust/pull/946).

## main

### Changed

- Update the Number in the SDK API to support min and max. #989

## v0.18.0

### Changed

- *BREAKING* `struct`s which implement `ShouldSample` a.k.a Custom Samplers must now
  implement `Clone`. This enables (#833)
- SDK split from `opentelemetry` crate
