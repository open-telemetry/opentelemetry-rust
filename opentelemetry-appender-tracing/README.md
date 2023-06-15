![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Log Appender for Tracing

A [Log
Appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge)
that bridges logs from the [tracing crate](https://tracing.rs/tracing/#events)
to OpenTelemetry logs. Note that this is different from the existing
[tracing-opentelemetry](https://github.com/tokio-rs/tracing-opentelemetry)
project, which supports bridging traces and logs from tracing into OpenTelemetry
traces. This is an experimental component, and could be merged with the
tracing-opentelemetry crate itself.

[![Crates.io: opentelemetry-appender-tracing](https://img.shields.io/crates/v/opentelemetry-appender-tracing.svg)](https://crates.io/crates/opentelemetry-appender-tracing)
[![Documentation](https://docs.rs/opentelemetry-appender-tracing/badge.svg)](https://docs.rs/opentelemetry-appender-tracing)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-appender-tracing)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to vendors or using experimental propagators like `base64`.

[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
