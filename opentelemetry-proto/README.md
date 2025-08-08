# OpenTelemetry Proto

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains generated Rust code from the
[opentelemetry-proto](https://github.com/open-telemetry/opentelemetry-proto)
repository and provides transformation functions between the generated protobuf
types and the types defined in the
[opentelemetry](https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry)
API crate.

[![Crates.io: opentelemetry-proto](https://img.shields.io/crates/v/opentelemetry-proto.svg)](https://crates.io/crates/opentelemetry-proto)
[![Documentation](https://docs.rs/opentelemetry-proto/badge.svg)](https://docs.rs/opentelemetry-proto)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-proto)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-proto/LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

*[Supported Rust Versions](#supported-rust-versions)*

## Getting started

See [docs](https://docs.rs/opentelemetry-proto).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-proto/CHANGELOG.md).

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.75.0. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.