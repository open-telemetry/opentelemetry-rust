![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Datadog

Community supported vendor integrations for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-datadog](https://img.shields.io/crates/v/opentelemetry-datadog.svg)](https://crates.io/crates/opentelemetry-datadog)
[![Documentation](https://docs.rs/opentelemetry-datadog/badge.svg)](https://docs.rs/opentelemetry-datadog)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-datadog)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to [`Datadog`].

## Features

`opentelemetry-datadog` supports following features:

- `reqwest-blocking-client`: use `reqwest` blocking http client to send spans.
- `reqwest-client`: use `reqwest` http client to send spans.
- `surf-client`: use `surf` http client to send spans.


## Kitchen Sink Full Configuration

 [Example](https://docs.rs/opentelemetry-datadog/latest/opentelemetry_datadog/#kitchen-sink-full-configuration) showing how to override all configuration options. See the
 [`DatadogPipelineBuilder`] docs for details of each option.

 [`DatadogPipelineBuilder`]: https://docs.rs/opentelemetry-datadog/latest/opentelemetry_datadog/struct.DatadogPipelineBuilder.html

[`Datadog`]: https://www.datadoghq.com/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
