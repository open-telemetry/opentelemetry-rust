# Changelog

## main

### Changed

- Update the Number in the SDK API to support min and max. #989

## v0.18.0

### Changed

- *BREAKING* `struct`s which implement `ShouldSample` a.k.a Custom Samplers must now
  implement `Clone`. This enables (#833)
- SDK split from `opentelemetry` crate
