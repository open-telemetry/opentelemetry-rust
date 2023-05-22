![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Log Appender

Community supported vendor integrations for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-contrib](https://img.shields.io/crates/v/opentelemetry-contrib.svg)](https://crates.io/crates/opentelemetry-contrib)
[![Documentation](https://docs.rs/opentelemetry-contrib/badge.svg)](https://docs.rs/opentelemetry-contrib)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-contrib)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to vendors or using experimental propagators like `base64`.

[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
