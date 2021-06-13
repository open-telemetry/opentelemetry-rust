# Changelog

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
