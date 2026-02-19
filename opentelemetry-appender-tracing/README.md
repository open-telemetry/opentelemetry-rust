# OpenTelemetry Log Appender for `tracing` crate

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains a [Log
Appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge)
that bridges logs from the [tracing crate](https://tracing.rs/tracing/#events)
to OpenTelemetry. Note that this is different from the existing
[tracing-opentelemetry](https://github.com/tokio-rs/tracing-opentelemetry)
project, which supports bridging traces and logs from tracing into OpenTelemetry
traces.

[![Crates.io: opentelemetry-appender-tracing](https://img.shields.io/crates/v/opentelemetry-appender-tracing.svg)](https://crates.io/crates/opentelemetry-appender-tracing)
[![Documentation](https://docs.rs/opentelemetry-appender-tracing/badge.svg)](https://docs.rs/opentelemetry-appender-tracing)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-appender-tracing)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-tracing/LICENSE)
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

*[Supported Rust Versions](#supported-rust-versions)*

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-tracing/CHANGELOG.md).

## Supported Rust Versions

[![MSRV](https://img.shields.io/crates/msrv/opentelemetry-appender-tracing)](https://crates.io/crates/opentelemetry-appender-tracing)
