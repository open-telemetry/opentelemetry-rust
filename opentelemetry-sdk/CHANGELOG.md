# Changelog

## vNext

## v0.22.1

### Fixed

- [#1576](https://github.com/open-telemetry/opentelemetry-rust/pull/1576)
  Fix Span kind is always set to "internal".

## v0.22.0

### Deprecated

- XrayIdGenerator in the opentelemetry-sdk has been deprecated and moved to version 0.10.0 of the opentelemetry-aws crate.

### Added

- [#1410](https://github.com/open-telemetry/opentelemetry-rust/pull/1410) Add experimental synchronous gauge
- [#1471](https://github.com/open-telemetry/opentelemetry-rust/pull/1471) Configure batch log record processor via [`OTEL_BLRP_*`](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/configuration/sdk-environment-variables.md#batch-logrecord-processor) environment variables and via `OtlpLogPipeline::with_batch_config`
- [#1503](https://github.com/open-telemetry/opentelemetry-rust/pull/1503) Make the documentation for In-Memory exporters visible.

- [#1526](https://github.com/open-telemetry/opentelemetry-rust/pull/1526)
Performance Improvement : Creating Spans and LogRecords are now faster, by avoiding expensive cloning of `Resource` for every Span/LogRecord.

### Changed
- **Breaking**
[#1313](https://github.com/open-telemetry/opentelemetry-rust/pull/1313)
[#1350](https://github.com/open-telemetry/opentelemetry-rust/pull/1350)
  Changes how Span links/events are stored to achieve performance gains. See
  below for details:

  *Behavior Change*: When enforcing `max_links_per_span`, `max_events_per_span`
  from `SpanLimits`, links/events are kept in the first-come order. The previous
  "eviction" based approach is no longer performed.

  *Breaking Change Affecting Exporter authors*:

  `SpanData` now stores `links` as `SpanLinks` instead of `EvictedQueue` where
  `SpanLinks` is a struct with a `Vec` of links and `dropped_count`.

  `SpanData` now stores `events` as `SpanEvents` instead of `EvictedQueue` where
  `SpanEvents` is a struct with a `Vec` of events and `dropped_count`.
- **Breaking** Remove `TextMapCompositePropagator` [#1373](https://github.com/open-telemetry/opentelemetry-rust/pull/1373). Use `TextMapCompositePropagator` in opentelemetry API.

- [#1375](https://github.com/open-telemetry/opentelemetry-rust/pull/1375/) Fix metric collections during PeriodicReader shutdown
- **Breaking** [#1480](https://github.com/open-telemetry/opentelemetry-rust/pull/1480) Remove fine grained `BatchConfig` configurations from `BatchLogProcessorBuilder` and `BatchSpanProcessorBuilder`. Use `BatchConfigBuilder` to construct a `BatchConfig` instance and pass it using `BatchLogProcessorBuilder::with_batch_config` or `BatchSpanProcessorBuilder::with_batch_config`.
- **Breaking** [#1480](https://github.com/open-telemetry/opentelemetry-rust/pull/1480) Remove mutating functions from `BatchConfig`, use `BatchConfigBuilder` to construct a `BatchConfig` instance.
- **Breaking** [#1495](https://github.com/open-telemetry/opentelemetry-rust/pull/1495) Remove Batch LogRecord&Span Processor configuration via non-standard environment variables. Use the following table to migrate from the no longer supported non-standard environment variables to the standard ones.

| No longer supported             | Standard equivalent       |
|---------------------------------|---------------------------|
| OTEL_BLRP_SCHEDULE_DELAY_MILLIS | OTEL_BLRP_SCHEDULE_DELAY  |
| OTEL_BLRP_EXPORT_TIMEOUT_MILLIS | OTEL_BLRP_EXPORT_TIMEOUT  |
| OTEL_BSP_SCHEDULE_DELAY_MILLIS  | OTEL_BSP_SCHEDULE_DELAY   |
| OTEL_BSP_EXPORT_TIMEOUT_MILLIS  | OTEL_BSP_EXPORT_TIMEOUT   |

- **Breaking** [1455](https://github.com/open-telemetry/opentelemetry-rust/pull/1455) Make the LoggerProvider Owned
  - `Logger` now takes an Owned Logger instead of a `Weak<LoggerProviderInner>`
  - `LoggerProviderInner` is no longer `pub (crate)`
  - `Logger.provider()` now returns `&LoggerProvider` instead of an `Option<LoggerProvider>`

- [1519](https://github.com/open-telemetry/opentelemetry-rust/pull/1519) Performance improvements
    when calling `Counter::add()` and `UpDownCounter::add()` with an empty set of attributes
    (e.g. `counter.Add(5, &[])`)

### Fixed

- [#1481](https://github.com/open-telemetry/opentelemetry-rust/pull/1481) Fix error message caused by race condition when using PeriodicReader

## v0.21.2

### Fixed

- Fix delta aggregation metric reuse. (#1434)
- Fix `max_scale` validation of exponential histogram configuration. (#1452)

## v0.21.1

### Fixed

- Fix metric export corruption if gauges have not received a last value. (#1363)
- Return consistent `Meter` for a given scope from `MeterProvider`. (#1351)

## v0.21.0

### Added

- Log warning if two instruments have the same name with different (#1266)
  casing
- Log warning if view is created with empty criteria (#1266)
- Add exponential histogram support (#1267)
- Add `opentelemetry::sdk::logs::config()` for parity with `opentelemetry::sdk::trace::config()` (#1197)

### Changed

- Renamed `MeterProvider` and `Meter` to `SdkMeterProvider` and `SdkMeter` respectively to avoid name collision with public API types. [#1328](https://github.com/open-telemetry/opentelemetry-rust/pull/1328)
- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
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
- Updated crate documentation and examples.
  [#1256](https://github.com/open-telemetry/opentelemetry-rust/issues/1256)
- Replace regex with glob (#1301)
- **Breaking**
  [#1293](https://github.com/open-telemetry/opentelemetry-rust/issues/1293)
  makes few breaking changes with respect to how Span attributes are stored to
  achieve performance gains. See below for details:

  *Behavior Change*:

  SDK will no longer perform de-duplication of Span attribute Keys. Please share
  [feedback
  here](https://github.com/open-telemetry/opentelemetry-rust/issues/1300), if
  you are affected.

  *Breaking Change Affecting Exporter authors*:

   `SpanData` now stores `attributes` as `Vec<KeyValue>` instead of
  `EvictedHashMap`. `SpanData` now expose `dropped_attributes_count` as a
  separate field.

  *Breaking Change Affecting Sampler authors*:

  `should_sample` changes `attributes` from `OrderMap<Key, Value>` to
  `Vec<KeyValue>`.
- **Breaking** Move type argument from `RuntimeChannel<T>` to associated types [#1314](https://github.com/open-telemetry/opentelemetry-rust/pull/1314)

### Removed

- Remove context from Metric force_flush [#1245](https://github.com/open-telemetry/opentelemetry-rust/pull/1245)
- Remove `logs::BatchMessage` and `trace::BatchMessage` types [#1314](https://github.com/open-telemetry/opentelemetry-rust/pull/1314)

### Fixed

- Fix metric instrument name validation to include `_` #1030

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
