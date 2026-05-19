# OpenTelemetry Stdout Exporter

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains an [OpenTelemetry](https://opentelemetry.io/) exporter that
prints telemetry (logs, metrics and traces) to the standard output.

[![Crates.io: opentelemetry-stdout](https://img.shields.io/crates/v/opentelemetry-stdout.svg)](https://crates.io/crates/opentelemetry-stdout)
[![Documentation](https://docs.rs/opentelemetry-stdout/badge.svg)](https://docs.rs/opentelemetry-stdout)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-stdout)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-stdout/LICENSE)
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

This crate includes exporters that support all three signals - Logs, Metrics,
and Traces — to standard output. It is intended solely for educational and
debugging purposes. Please note, this crate is not optimized for performance,
and the format of the output may change, making it unsuitable for production
environments

## Getting started

See [docs](https://docs.rs/opentelemetry-stdout).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-stdout/CHANGELOG.md).

## Supported Rust Versions

[![MSRV](https://img.shields.io/crates/msrv/opentelemetry-stdout)](https://crates.io/crates/opentelemetry-stdout)
