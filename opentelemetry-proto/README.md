# OpenTelemetry Proto

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains generated files from
[opentelemetry-proto](https://github.com/open-telemetry/opentelemetry-proto)
repository and transformations between types from generated files and types
defined in
[opentelemetry](https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry).

[![Crates.io: opentelemetry-proto](https://img.shields.io/crates/v/opentelemetry-proto.svg)](https://crates.io/crates/opentelemetry-proto)
[![Documentation](https://docs.rs/opentelemetry-proto/badge.svg)](https://docs.rs/opentelemetry-proto)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-proto)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-proto/LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![codecov](https://codecov.io/gh/open-telemetry/opentelemetry-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/open-telemetry/opentelemetry-rust)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

OpenTelemetry is an Observability framework and toolkit designed to create and
manage telemetry data such as traces, metrics, and logs. OpenTelemetry is
vendor- and tool-agnostic, meaning that it can be used with a broad variety of
Observability backends, including open source tools like [Jaeger] and
[Prometheus], as well as commercial offerings.

OpenTelemetry is *not* an observability backend like Jaeger, Prometheus, or other
commercial vendors. OpenTelemetry is focused on the generation, collection,
management, and export of telemetry. A major goal of OpenTelemetry is that you
can easily instrument your applications or systems, no matter their language,
infrastructure, or runtime environment. Crucially, the storage and visualization
of telemetry is intentionally left to other tools.

*[Supported Rust Versions](#supported-rust-versions)*

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io

### What does this crate contain?

This crate provides auto-generated Protobuf types from the [OpenTelemetry
protocol specification](https://github.com/open-telemetry/opentelemetry-proto),
along with conversion implementations between these generated types and the
types defined in the
[opentelemetry](https://crates.io/crates/opentelemetry) crate. It is used
internally by exporters such as
[opentelemetry-otlp](https://crates.io/crates/opentelemetry-otlp) to serialize
and deserialize telemetry data.

## Getting started

See [docs](https://docs.rs/opentelemetry-proto).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-proto/CHANGELOG.md).

## Supported Rust Versions

[![MSRV](https://img.shields.io/crates/msrv/opentelemetry-proto)](https://crates.io/crates/opentelemetry-proto)
