# OpenTelemetry Rust

This crate contains the [OpenTelemetry](https://opentelemetry.io/) API for Rust.

[![Crates.io: opentelemetry](https://img.shields.io/crates/v/opentelemetry.svg)](https://crates.io/crates/opentelemetry)
[![Documentation](https://docs.rs/opentelemetry/badge.svg)](https://docs.rs/opentelemetry)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![codecov](https://codecov.io/gh/open-telemetry/opentelemetry-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/open-telemetry/opentelemetry-rust)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

OpenTelemetry is an Observability framework and toolkit designed to create and
manage telemetry data such as traces, metrics, and logs. Crucially,
OpenTelemetry is vendor- and tool-agnostic, meaning that it can be used with a
broad variety of Observability backends, including open source tools like
[Jaeger] and [Prometheus], as well as commercial offerings.

OpenTelemetry is not an observability backend like Jaeger, Prometheus, or other
commercial vendors. OpenTelemetry is focused on the generation, collection,
management, and export of telemetry. A major goal of OpenTelemetry is that you
can easily instrument your applications or systems, no matter their language,
infrastructure, or runtime environment. Crucially, the storage and visualization
of telemetry is intentionally left to other tools.

*Compiler support: [requires `rustc` 1.64+][msrv]*

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io
[msrv]: #supported-rust-versions

### What does this crate contain?

This is the API crate which contains the [Context
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/context/README.md),
[Propagators
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/context/api-propagators.md),[Baggage
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/baggage/api.md),
[Logs Bridge
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/bridge-api.md),
[Tracing
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/api.md),
and [Metrics
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md).

This crates allows one to instrument libraries and application, but the APIs in
this crate are ["no-ops" or just
facades](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/library-guidelines.md#api-and-minimal-implementation),
with actual implementation occurring in the
[opentelemetry-sdk](https://crates.io/crates/opentelemetry-sdk). This crate does
not deal with concepts such as processing or exporting of telemetry, they are
handled by the [opentelemetry-sdk](https://crates.io/crates/opentelemetry-sdk),
along with one or more exporters like the
[opentelemetry-otlp](https://crates.io/crates/opentelemetry-otlp).

If you are a library author, then this is the crate you should be using to
instrument it. If you are familiar with `tracing` or `log` ecosystem, this crate
is just the facade part, and gets lights up, when the final application uses an
sdk implementation.

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.64. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.
