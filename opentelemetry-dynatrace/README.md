![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Dynatrace

[`Dynatrace`] integration for applications instrumented with [`OpenTelemetry`].

[![Crates.io: opentelemetry-dynatrace](https://img.shields.io/crates/v/opentelemetry-dynatrace.svg)](https://crates.io/crates/opentelemetry-dynatrace)
[![Documentation](https://docs.rs/opentelemetry-dynatrace/badge.svg)](https://docs.rs/opentelemetry-dynatrace)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry-dynatrace)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

## Overview

[`OpenTelemetry`] is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. This
crate provides additional propagators and exporters for sending telemetry data
to [`Dynatrace`].

## Exporter features

* **Metrics** - Ingest metric data to Dynatrace using the [Dynatrace Metrics ingestion protocol].

This exporter only supports the ingestion of metric data. For trace data, use 
[`opentelemetry-otlp`] as described in the 
[Dynatrace documentation for Rust]. This exporter is based on the OpenTelemetry 
Metrics SDK for Rust, which is currently in an alpha state and neither 
considered stable nor complete as of this writing. As such, this exporter is 
not intended for production use until the underlying OpenTelemetry Metrics API 
and SDK are stable. See [`open-telemetry/opentelemetry-rust`] for the current 
state of the OpenTelemetry SDK for Rust.

[Dynatrace]: https://www.dynatrace.com/
[Dynatrace Metrics ingestion protocol]: https://www.dynatrace.com/support/help/how-to-use-dynatrace/metrics/metric-ingestion/metric-ingestion-protocol/
[Dynatrace documentation for Rust]: https://www.dynatrace.com/support/help/extend-dynatrace/opentelemetry/opentelemetry-ingest/opent-rust/
[`open-telemetry/opentelemetry-rust`]: https://github.com/open-telemetry/opentelemetry-rust

### Examples

The examples directory contains an [advanced example](../examples/dynatrace) 
showing the ingestion of trace data and metric data together.

[`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
[`opentelemetry-dynatrace`]: https://crates.io/crates/opentelemetry-dynatrace

## Performance

For optimal performance, a batch exporter is used. You can enable the `rt-tokio` 
feature flag to use the [`tokio`] runtime, or enable the `rt-async-std` feature 
flag to use the [`async-std`] runtime to have a batch exporter configured for 
you automatically.

[`tokio`]: https://tokio.rs
[`async-std`]: https://async.rs

## Choosing an HTTP client

The HTTP client that this exporter will use can be overridden with feature 
flags. By default the `reqwest-client` feature flag is enabled which will use 
the [`reqwest`] http client.

- `reqwest-client` (enabled by default): use the [`reqwest`] http client to send metric data.
- `reqwest-tls` (enabled by default): use the [`reqwest`] http client with [`rustls`] to enable TLS support.
- `reqwest-blocking-client`: use the [`reqwest`] blocking http client to send metric data.
- `isahc-client`: use the [`isahc`] http client to send metric data.
- `surf-client`: use the [`surf`] http client to send metric data.

You can also configure your own http client implementation using the `HttpClient` trait.

[`reqwest`]: https://docs.rs/reqwest/latest/reqwest/
[`rustls`]: https://docs.rs/rustls/latest/rustls/
[`isahc`]: https://docs.rs/isahc/latest/isahc/
[`surf`]: https://docs.rs/surf/latest/surf/

## WebAssembly

WebAssembly support can be enabled with the `wasm` feature flag.

[`Dynatrace`]: https://www.dynatrace.com/
[`OpenTelemetry`]: https://crates.io/crates/opentelemetry
