# OpenTelemetry Jaeger Propagator

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains [`Jaeger`] propagator integration for applications
instrumented with [`OpenTelemetry`]. To export telemetry to Jaeger, use the
opentelemetry-otlp crate.

[![Crates.io: opentelemetry-jaeger-propagator](https://img.shields.io/crates/v/opentelemetry-jaeger-propagator.svg)](https://crates.io/crates/opentelemetry-jaeger-propagator)
[![Documentation](https://docs.rs/opentelemetry-jaeger-propagator/badge.svg)](https://docs.rs/opentelemetry-jaeger-propagator)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-jaeger-propagator)](./LICENSE)
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
