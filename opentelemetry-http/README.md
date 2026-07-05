# OpenTelemetry HTTP

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains helper implementations for sending HTTP requests, used
by [OpenTelemetry](https://opentelemetry.io/) components. Common uses include
propagating and extracting context over HTTP, exporting telemetry data over
HTTP, and requesting sampling strategies from remote endpoints.

[![Crates.io: opentelemetry-http](https://img.shields.io/crates/v/opentelemetry-http.svg)](https://crates.io/crates/opentelemetry-http)
[![Documentation](https://docs.rs/opentelemetry-http/badge.svg)](https://docs.rs/opentelemetry-http)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-http)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-http/LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

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

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io

### What does this crate contain?

This crate contains shared HTTP building blocks used across the
OpenTelemetry Rust ecosystem, including:

- An `HttpClient` trait and adapters for popular HTTP clients such as
  `reqwest` and `hyper`.
- Helpers for propagating and extracting OpenTelemetry context (for example
  W3C TraceContext) over HTTP headers.
- Utilities used by exporters to send telemetry over HTTP.

This crate is primarily used as a dependency by other OpenTelemetry crates
(such as [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp))
rather than directly by end users.

### Related crates

- **[opentelemetry](https://crates.io/crates/opentelemetry):** The core
  OpenTelemetry API.
- **[opentelemetry-sdk](https://crates.io/crates/opentelemetry-sdk):** The
  OpenTelemetry SDK implementation.
- **[opentelemetry-otlp](https://crates.io/crates/opentelemetry-otlp):** OTLP
  exporter that uses this crate for HTTP transport.

## Getting started

See [docs](https://docs.rs/opentelemetry-http).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-http/CHANGELOG.md).

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.75.0. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.
