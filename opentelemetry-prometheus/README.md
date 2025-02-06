# OpenTelemetry Prometheus Exporter

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

[`Prometheus`] integration for applications instrumented with [`OpenTelemetry`]. 

**The development of prometheus exporter has halt until the Opentelemetry metrics API and SDK reaches 1.0. Current 
implementation is based on Opentelemetry API and SDK 0.27**.

[![Crates.io: opentelemetry-prometheus](https://img.shields.io/crates/v/opentelemetry-prometheus.svg)](https://crates.io/crates/opentelemetry-prometheus)
[![Documentation](https://docs.rs/opentelemetry-prometheus/badge.svg)](https://docs.rs/opentelemetry-prometheus)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-prometheus)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## OpenTelemetry Overview

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

[`Prometheus`]: https://prometheus.io
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry

## Release Notes

You can find the release notes (changelog) [here](./CHANGELOG.md).
