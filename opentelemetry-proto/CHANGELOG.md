# Changelog

## vNext

## v0.5.0

### Changed

- Update to tonic 0.11 and prost 0.12 (#1536)
- **Breaking** Remove support for grpcio transport (#1534)

### Added

- Add `schemars::JsonSchema` trait support with `with-schemars` feature (#1419)
- Update protobuf definitions to v1.1.0 (#1154)

## v0.4.0

### Added

- Implement tonic metrics proto transformations (#1184)
- Move proto for zPage to tonic [#1214](https://github.com/open-telemetry/opentelemetry-rust/pull/1214)
- Support exponential histograms (#1267)

### Changed

- Bump MSRV to 1.65 [#1318](https://github.com/open-telemetry/opentelemetry-rust/pull/1318)
- Bump MSRV to 1.64 [#1203](https://github.com/open-telemetry/opentelemetry-rust/pull/1203)

### Fixed

- Rename `traces` feature to the more standard `trace` (#1183)

### Changed

- Switch to `prost` for `grpcio` protos. (#1202)
  The `gen-protoc` feature is accordingly renamed to `gen-grpcio`.

## v0.3.0

### Updated

- Update protobuf definitions to v1.0.0 #1154
- Update to opentelemetry-api v0.20.0

## v0.2.0
### Changed
- Update to opentelemetry v0.19.0.
- Remove build script and generate files using unit tests [#881](https://github.com/open-telemetry/opentelemetry-rust/pull/881).
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953).
- Update dependencies and bump MSRV to 1.60 [#969](https://github.com/open-telemetry/opentelemetry-rust/pull/969).
- Bump to use the v0.19.0 protobuf definitions. [#989](https://github.com/open-telemetry/opentelemetry-rust/pull/989).

## v0.1.0

Initial crate release.
