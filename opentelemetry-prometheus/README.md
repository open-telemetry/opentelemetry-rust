![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo-text.png

# OpenTelemetry Prometheus 

[`Prometheus`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-prometheus](https://img.shields.io/crates/v/opentelemetry-prometheus.svg)](https://crates.io/crates/opentelemetry-prometheus)
[![Documentation](https://docs.rs/opentelemetry-prometheus/badge.svg)](https://docs.rs/opentelemetry-prometheus)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-prometheus)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amaster)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-prometheus) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides a pipeline and exporter for exposing metrics information to
Prometheus for processing and visualization.

[`Prometheus`]: https://prometheus.io
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
