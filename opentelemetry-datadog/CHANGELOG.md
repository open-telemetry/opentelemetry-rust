# Changelog

## v0.7.0
### Added
- [Breaking] Add support for unified tagging [#931](https://github.com/open-telemetry/opentelemetry-rust/pull/931).

### Changed
- Update `opentelemetry` to 0.19
- Update `opentelemetry-http` to 0.8
- Update `opentelemetry-semantic-conventions` to 0.11.
- Bump MSRV to 1.57 [#953](https://github.com/open-telemetry/opentelemetry-rust/pull/953)
- Send resource with attributes [#880](https://github.com/open-telemetry/opentelemetry-rust/pull/880).
- Update msgpack accounting for sampling_priority [#903](https://github.com/open-telemetry/opentelemetry-rust/pull/903).

## v0.6.0

### Changed

- Allow custom mapping #770
- Update to opentelemetry v0.18.0
- Update to opentelemetry-http v0.7.0
- Update to opentelemetry-semantic-conventions v0.10.0
- Parse config endpoint to remove tailing slash #787
- Add sampling priority tag in spans #792

## v0.5.0

### Changed

- Update to opentelemetry v0.17.0
- Update to opentelemetry-http v0.6.0
- Update to opentelemetry-semantic-conventions v0.9.0

## v0.4.0

### Changed

- Update to opentelemetry v0.16.0

## v0.3.1

### Fixed

- `status_code` must be 0 or 1 #580

## v0.3.0

### Changed

- Update to opentelemetry v0.15.0

## v0.2.0

### Changed

- Disable optional features for reqwest
- Remove default surf features #546
- Update to opentelemetry v0.14.0

## v0.1.0

### Added

- Datadog exporter #446
- Datadog propagator #440

### Changed
- Rename trace config with_default_sampler to with_sampler #482
