# Changelog

## vNext

## v0.17.0

- Add "metrics", "logs" to default features. With this, default feature list is
  "trace", "metrics" and "logs".
- `OtlpMetricPipeline.build()` no longer invoke the
  `global::set_meter_provider`. User who setup the pipeline must do it
  themselves using `global::set_meter_provider(meter_provider.clone());`.
- Add `with_resource` on `OtlpLogPipeline`, replacing the `with_config` method.
Instead of using
`.with_config(Config::default().with_resource(RESOURCE::default()))` users must
now use `.with_resource(RESOURCE::default())` to configure Resource when using
`OtlpLogPipeline`.
- **Breaking** The methods `OtlpTracePipeline::install_simple()` and `OtlpTracePipeline::install_batch()` would now return `TracerProvider` instead of `Tracer`.
  These methods would also no longer set the global tracer provider. It would now be the responsibility of users to set it by calling `global::set_tracer_provider(tracer_provider.clone());`. Refer to the [basic-otlp](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp/src/main.rs) and [basic-otlp-http](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs) examples on how to initialize OTLP Trace Exporter.
- **Breaking** Correct the misspelling of "webkpi" to "webpki" in features [#1842](https://github.com/open-telemetry/opentelemetry-rust/pull/1842)
- Bump MSRV to 1.70 [#1840](https://github.com/open-telemetry/opentelemetry-rust/pull/1840)
- Fixing the OTLP HTTP/JSON exporter. [#1882](https://github.com/open-telemetry/opentelemetry-rust/pull/1882) - The exporter was broken in the
  previous release.
- **Breaking** [1869](https://github.com/open-telemetry/opentelemetry-rust/pull/1869) The OTLP logs exporter now overrides the [InstrumentationScope::name](https://github.com/open-telemetry/opentelemetry-proto/blob/b3060d2104df364136d75a35779e6bd48bac449a/opentelemetry/proto/common/v1/common.proto#L73) field with the `target` from `LogRecord`, if target is populated.
- Groups batch of `LogRecord` and `Span` by their resource and instrumentation scope before exporting, for better efficiency [#1873](https://github.com/open-telemetry/opentelemetry-rust/pull/1873).
- **Breaking** Update to `http` v1 and `tonic` v0.12 [#1674](https://github.com/open-telemetry/opentelemetry-rust/pull/1674)
- Update `opentelemetry` dependency version to 0.24
- Update `opentelemetry_sdk` dependency version to 0.24
- Update `opentelemetry-http` dependency version to 0.13
- Update `opentelemetry-proto` dependency version to 0.7

## v0.16.0

### Fixed

- URL encoded values in `OTEL_EXPORTER_OTLP_HEADERS` are now correctly decoded. [#1578](https://github.com/open-telemetry/opentelemetry-rust/pull/1578)
- OTLP exporter will not change the URL added through `ExportConfig` [#1706](https://github.com/open-telemetry/opentelemetry-rust/pull/1706)
- Default grpc endpoint will not have path based on signal(e.g `/v1/traces`) [#1706](https://github.com/open-telemetry/opentelemetry-rust/pull/1706)
- Fix feature flags for `OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT` [#1746](https://github.com/open-telemetry/opentelemetry-rust/pull/1746)

### Added

- Added `DeltaTemporalitySelector` ([#1568])
- Add `webkpi-roots` features to `reqwest` and `tonic` backends

[#1568]: https://github.com/open-telemetry/opentelemetry-rust/pull/1568

### Changed
 - **Breaking** Remove global provider for Logs [#1691](https://github.com/open-telemetry/opentelemetry-rust/pull/1691/)
      - The method OtlpLogPipeline::install_simple() and OtlpLogPipeline::install_batch() now return `LoggerProvider` instead of
      `Logger`. Refer to the [basic-otlp](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp/src/main.rs) and [basic-otlp-http](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs) examples for how to initialize OTLP Log Exporter to use with OpenTelemetryLogBridge and OpenTelemetryTracingBridge respectively.
- Update `opentelemetry` dependency version to 0.23
- Update `opentelemetry_sdk` dependency version to 0.23
- Update `opentelemetry-http` dependency version to 0.12
- Update `opentelemetry-proto` dependency version to 0.6

## v0.15.0

### Added

- Support custom channels in topic exporters  [#1335](https://github.com/open-telemetry/opentelemetry-rust/pull/1335)
- Allow specifying OTLP Tonic metadata from env variable [#1377](https://github.com/open-telemetry/opentelemetry-rust/pull/1377)

### Changed
- Update to tonic 0.11 and prost 0.12 [#1536](https://github.com/open-telemetry/opentelemetry-rust/pull/1536)

### Fixed
- Fix `tonic()` to the use correct port. [#1556](https://github.com/open-telemetry/opentelemetry-rust/pull/1556)

### Removed
- **Breaking** Remove support for surf HTTP client [#1537](https://github.com/open-telemetry/opentelemetry-rust/pull/1537)
- **Breaking** Remove support for grpcio transport [#1534](https://github.com/open-telemetry/opentelemetry-rust/pull/1534)

## v0.14.0

### Added

- Add `build_{signal}_exporter` methods to client builders [#1187](https://github.com/open-telemetry/opentelemetry-rust/pull/1187)
- Add `grpcio` metrics exporter [#1202](https://github.com/open-telemetry/opentelemetry-rust/pull/1202)
- Allow specifying OTLP HTTP headers from env variable [#1290](https://github.com/open-telemetry/opentelemetry-rust/pull/1290)

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)
- Changed dependency from `opentelemetry_api` to `opentelemetry` as the latter
  is now the API crate. [#1226](https://github.com/open-telemetry/opentelemetry-rust/pull/1226)
- Make `NoExporterBuilder` a compiling time error [#1272](https://github.com/open-telemetry/opentelemetry-rust/pull/1272)

## v0.13.0

### Added

- Add OTLP HTTP Metrics Exporter [#1020](https://github.com/open-telemetry/opentelemetry-rust/pull/1020).
- Add tonic compression support [#1165](https://github.com/open-telemetry/opentelemetry-rust/pull/1165).

### Changed

- make the tonic/transport feature optional #985
- update to opentelemetry-api v0.20.0

### Fixed

- Fix a missing import when http-proto is enabled without grpc-sys #1081

## v0.12.0

### Added

- Add batch config for otlp pipeline [#979](https://github.com/open-telemetry/opentelemetry-rust/pull/979).
- Add tonic interceptor [#901](https://github.com/open-telemetry/opentelemetry-rust/pull/901).

### Changed

- Update `opentelemetry` to 0.19.
- Update `opentelemetry-semantic-conventions` to 0.11.
- Update `opentelemetry-http` to 0.8.
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Add `User-Agent` header on all exporters [#896](https://github.com/open-telemetry/opentelemetry-rust/pull/896).
- Improve OTLP exporter environment variable handling [#912](https://github.com/open-telemetry/opentelemetry-rust/pull/912).
- Fix the issue where tonic exporter builder ignored provided metadata [#937](https://github.com/open-telemetry/opentelemetry-rust/pull/937).
- Export `MetricsExporterBuilder` [#943](https://github.com/open-telemetry/opentelemetry-rust/pull/943).
- Report OTLP http export errors [#945](https://github.com/open-telemetry/opentelemetry-rust/pull/945).
- Change to export using v0.19.0 protobuf definitions. [#989](https://github.com/open-telemetry/opentelemetry-rust/pull/989).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).


## v0.11.0

### Changed

- reduce `tokio` feature requirements #750
- Update to opentelemetry v0.18.0
- Update to opentelemetry-http v0.7.0
- Update `tonic` to 0.7 #783
- Automatically add traces / metrics paths #806

## v0.10.0

### Changed

- Update to opentelemetry v0.17.0
- Update to opentelemetry-http v0.6.0
- Update `tonic` to 0.6 #660

## v0.9.0

### Changed

- Merge metrics and tracing pipeline #585
- Update to opentelemetry v0.16.0

### Fixed

- `MetricsExporterBuilder` drops `exporter_pipeline` #590
- Improve error messages #603
- Upgrade `tonic` to `0.5.x` #597

## v0.8.0

### Changed

- Update grpcio version and add the coverage badge #556
- Update to opentelemetry v0.15.0

## v0.7.0

### Added

- adding otlp http transport, using proto binary #516

### Fixed

- docs cannot compile #507
- exporter cannot merge IntSum correctly. #518
- update metrics proto and metric transformation #535

### Changed

- Allow users to bring their own tonic channel  #515
- Remove default surf features #546
- Update to opentelemetry v0.14.0

### v0.6.0

### Added
- Examples on how to connect to an external otlp using tonic, tls and tokio #449
- Examples on how to connect to an external otlp using grpcio and tls #450
- `with_env` method for `OtlpPipelineBuilder` to use environment variables to config otlp pipeline #451
- Update `tracing-grpc` example to include extractors and injectors #464
- Mentioned `service.name` resource in README #476

### Changed
- Update to opentelemetry v0.13.0
- Update `tonic-build` dependency to 0.4 #463
- Update the opentelemetry pipeline to use API to choose grpc layer instead of feature #467
- Rename trace config with_default_sampler to with_sampler #482

### Removed
- Removed `from_env` and use environment variables to initialize the configurations by default #459
- Removed support for running tonic without tokio runtime #483

## v0.5.0

### Added

- Otlp metric exporter #402
- Otlp exporter integration test #424

### Changed

- Update to opentelemetry v0.12.0
- Update tokio to v1 #421

## v0.4.0

### Added

- Tonic support #352
- Add openssl feature flags for grpcio #367

### Changed

- Update to opentelemetry v0.11.0
- Update default otlp port to `4317` #388

### Fixed

- Propagate `Resource` information #366
- Propagate `Resource` in tonic as well #390

## v0.3.0

### Changed

- Update to opentelemetry v0.10.0

## v0.2.0

### Changed

- Update to opentelemetry v0.9.0
- Add exporter pipeline #210

## v0.1.0

### Added

- Initial Alpha implementation
