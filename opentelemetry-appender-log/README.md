# OpenTelemetry Log Appender for `log` crate

![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

This crate contains a [Log Appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) that bridges logs from the [log crate](https://docs.rs/log/latest/log/) to OpenTelemetry.

[![Crates.io: opentelemetry-appender-log](https://img.shields.io/crates/v/opentelemetry-appender-log.svg)](https://crates.io/crates/opentelemetry-appender-log)
[![Documentation](https://docs.rs/opentelemetry-appender-log/badge.svg)](https://docs.rs/opentelemetry-appender-log)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-appender-log)](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-log/LICENSE)
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

This crate provides a [Log Appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge)
that forwards log records emitted through the
[`log`](https://docs.rs/log/latest/log/) crate to the OpenTelemetry Logs
pipeline. Use this crate when your application (or its dependencies) already
use `log` and you want those records to be captured, enriched with
OpenTelemetry context, and exported through an OpenTelemetry exporter.

## Getting started

See [docs](https://docs.rs/opentelemetry-appender-log).

## Release Notes

You can find the release notes (changelog) [here](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-appender-log/CHANGELOG.md).

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.75.0. The current OpenTelemetry version is NOT guaranteed to build
on Rust versions earlier than the minimum supported version.

API and SDK-independent crates retain their existing MSRV for as long as
practical. MSRV bumps ship only in minor releases and are not considered
semver-breaking changes. See the [MSRV
policy](../docs/adr/002_msrv_policy.md) for the complete rules.