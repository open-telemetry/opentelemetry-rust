# Changelog

## vNext

## v0.14.0

### Added

- Add `build_{signal}_exporter` methods to client builders (#1187)
- Add `grpcio` metrics exporter (#1202)
- Allow specifying OTLP HTTP headers from env variable (#1290)
- Support custom channels in topic exporters  [#1335](https://github.com/open-telemetry/opentelemetry-rust/pull/1335)
- Allow specifying OTLP Tonic metadata from env variable (#1377)

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)
- Changed dependency from `opentelemetry_api` to `opentelemetry` as the latter
  is now the API crate. [#1226](https://github.com/open-telemetry/opentelemetry-rust/pull/1226)
- Make `NoExporterBuilder` a compiling time error [#1271](https://github.com/open-telemetry/opentelemetry-rust/pull/1271)

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
