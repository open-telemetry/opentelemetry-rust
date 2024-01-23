![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Jaeger Propagator

[`Jaeger`] propagator integration for applications instrumented with [`OpenTelemetry`]. Jaeger exporter functionality can be found in the `opentelemetry-jaeger` crate until this functionality is deprecated.

[![Crates.io: opentelemetry-jaeger](https://img.shields.io/crates/v/opentelemetry-jaeger.svg)](https://crates.io/crates/opentelemetry-jaeger)
[![Documentation](https://docs.rs/opentelemetry-jaeger/badge.svg)](https://docs.rs/opentelemetry-jaeger)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-jaeger)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides the ability to create and interact with a Jaeger propagator.

*Compiler support: [requires `rustc` 1.64+][msrv]*

[`Jaeger`]: https://www.jaegertracing.io/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
[msrv]: #supported-rust-versions

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.64. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.
