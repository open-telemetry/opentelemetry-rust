![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry derive macros

Derive macros for [`OpenTelemetry`].

[![Crates.io: opentelemetry-derive](https://img.shields.io/crates/v/opentelemetry-derive.svg)](https://crates.io/crates/opentelemetry-derive)
[![Documentation](https://docs.rs/opentelemetry-derive/badge.svg)](https://docs.rs/opentelemetry-derive)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-derive)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior.

This crate provides derive macros, it should not be used directly,
but through the `derive` feature of [`OpenTelemetry`].

[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
