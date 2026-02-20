# OpenTelemetry OTLP Exporter

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains the [OpenTelemetry](https://opentelemetry.io/) Exporter
implementation for
[OTLP](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md).

[![Crates.io: opentelemetry-otlp](https://img.shields.io/crates/v/opentelemetry-otlp.svg)](https://crates.io/crates/opentelemetry-otlp)
[![Documentation](https://docs.rs/opentelemetry-otlp/badge.svg)](https://docs.rs/opentelemetry-otlp)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-otlp)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/LICENSE)
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

## Getting started

See [docs](https://docs.rs/opentelemetry-otlp).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/CHANGELOG.md).

## Supported Rust Versions

[![MSRV](https://img.shields.io/crates/msrv/opentelemetry-otlp)](https://crates.io/crates/opentelemetry-otlp)
