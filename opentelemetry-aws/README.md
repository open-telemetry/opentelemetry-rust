![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry AWS

Community supported vendor integrations for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-aws](https://img.shields.io/crates/v/opentelemetry-aws.svg)](https://crates.io/crates/opentelemetry-aws)
[![Documentation](https://docs.rs/opentelemetry-aws/badge.svg)](https://docs.rs/opentelemetry-aws)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-aws)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

[Documentation](https://docs.rs/opentelemetry-aws) |
[Chat](https://gitter.im/open-telemetry/opentelemetry-rust)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to AWS's telemetry platform.

## Supported component

Currently, this crate only supports `XRay` propagator. Contributions are welcome.

[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
