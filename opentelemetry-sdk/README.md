![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Rust SDK

The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.

[![Crates.io: opentelemetry-sdk](https://img.shields.io/crates/v/opentelemetry_sdk.svg)](https://crates.io/crates/opentelemetry_sdk)
[![Documentation](https://docs.rs/opentelemetry_sdk/badge.svg)](https://docs.rs/opentelemetry_sdk)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry_sdk)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![codecov](https://codecov.io/gh/open-telemetry/opentelemetry-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/open-telemetry/opentelemetry-rust)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

OpenTelemetry is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. You
can export and analyze them using [Prometheus], [Jaeger], and other
observability tools.

*Compiler support: [requires `rustc` 1.64+][msrv]*

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io
[msrv]: #supported-rust-versions

## OpenTelemetry Benchmarks

From the root folder, run the following command:

```sh
cargo bench
```
