# Changelog

## vNext

### Added

- Log warning if two instruments have the same name with different (#1266)
  casing
- Log warning if view is created with empty criteria (#1266)

### Changed

- Default Resource (the one used when no other Resource is explicitly provided) now includes `TelemetryResourceDetector`,
  populating "telemetry.sdk.*" attributes.
  [#1066](https://github.com/open-telemetry/opentelemetry-rust/pull/1193).
- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)
- Add unit/doc tests for MeterProvider #1220
- Changed dependency from `opentelemetry_api` to `opentelemetry` as the latter
  is now the API crate. [#1226](https://github.com/open-telemetry/opentelemetry-rust/pull/1226)
- Add in memory span exporter [#1216](https://github.com/open-telemetry/opentelemetry-rust/pull/1216)
- Add in memory log exporter [#1231](https://github.com/open-telemetry/opentelemetry-rust/pull/1231)
- Add `Sync` bound to the `SpanExporter` and `LogExporter` traits [#1240](https://github.com/open-telemetry/opentelemetry-rust/pull/1240)
- Move `MetricsProducer` config to builders to match other config (#1266)
- Return error earlier if readers are shut down (#1266)
- Add `/` to valid characters for instrument names (#1269)
- Increase instrument name maximum length from 63 to 255 (#1269)

### Removed

- Remove context from Metric force_flush [#1245](https://github.com/open-telemetry/opentelemetry-rust/pull/1245)


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
