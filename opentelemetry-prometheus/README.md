# OpenTelemetry Prometheus Exporter

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

[`Prometheus`] integration for applications instrumented with [`OpenTelemetry`]. 

**Warning: This crate is no longer recommended for use.**

Development of the Prometheus exporter has been discontinued. See the related
[issue](https://github.com/open-telemetry/opentelemetry-rust/issues/2451). This
crate depends on the unmaintained `protobuf` crate and has unresolved security
vulnerabilities. Version 0.29 will be the final release.

For Prometheus integration, we strongly recommend using the [OTLP] exporter
instead. Prometheus [natively supports
OTLP](https://prometheus.io/docs/guides/opentelemetry/#enable-the-otlp-receiver),
providing a more stable and actively maintained solution.

[OTLP]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/

[![Crates.io: opentelemetry-prometheus](https://img.shields.io/crates/v/opentelemetry-prometheus.svg)](https://crates.io/crates/opentelemetry-prometheus)
[![Documentation](https://docs.rs/opentelemetry-prometheus/badge.svg)](https://docs.rs/opentelemetry-prometheus)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-prometheus)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-prometheus/LICENSE)
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

[`Prometheus`]: https://prometheus.io
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry

### What does this crate contain?

**⚠️ This crate is deprecated and no longer recommended for use.**

This crate previously provided direct Prometheus metrics export functionality, but has been discontinued due to:
- Dependency on unmaintained `protobuf` crate
- Unresolved security vulnerabilities
- Limited maintenance resources

For new projects, use the [OTLP exporter](https://docs.rs/opentelemetry-otlp/) instead, as Prometheus now natively supports OTLP.

## Getting started

**For new projects:** Use the [opentelemetry-otlp](https://docs.rs/opentelemetry-otlp/) crate instead. See the [Prometheus OTLP documentation](https://prometheus.io/docs/guides/opentelemetry/#enable-the-otlp-receiver) for integration details.

**For existing projects:** See the [docs](https://docs.rs/opentelemetry-prometheus) for legacy API reference, but plan migration to OTLP.

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-prometheus/CHANGELOG.md).

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
