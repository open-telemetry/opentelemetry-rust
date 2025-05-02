# Migration Guide from 0.28 to 0.29

OpenTelemetry Rust 0.29 introduces a few breaking changes. This guide aims to
facilitate a smooth migration for common use cases involving the
`opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, and
`opentelemetry-appender-tracing` crates. For a comprehensive list of changes,
please refer to the detailed changelog for each crate. This document covers only
the most common scenarios. Note that changes that only affect custom
exporter/processor authors are not mentioned in this doc.

OpenTelemetry Metrics API and Log-Bridge API were declared stable in 0.28, and have
no breaking changes.

## Baggage Changes

The Baggage API has been redesigned to align with the OpenTelemetry
specification. While the core API for interacting with Baggage remains the same,
the accepted data types have changed. Baggage Keys now only allow strings (ASCII
printable characters), and Baggage values are restricted to strings.

For detailed changes, see the [changelog](../opentelemetry/CHANGELOG.md). With
version 0.29, the Baggage API has reached "Release Candidate" status, meaning
further breaking changes will be highly restricted.

## Appender-Tracing Changes

The `opentelemetry-appender-tracing` crate, which bridges `tracing` events to
OpenTelemetry logs, has been updated to properly map `tracing` data types to the
OpenTelemetry model. As of version 0.29, this crate is considered "Stable," and
no further breaking changes will be made without a major version bump.

## Configuration via Environment Variables

The 0.29 release aligns OpenTelemetry Rust with the rest of the OpenTelemetry
ecosystem by treating any code-based configuration as final (i.e., it cannot be
overridden by environment variables). This policy was partially true before but
is now applied consistently. If you prefer to configure your application via
environment variables, avoid configuring it programmatically.

## Discontinuing Dedicated Prometheus Exporter

The `opentelemetry-prometheus` crate will be discontinued with the 0.29 release.
Active development on this crate ceased a few months ago. Given that Prometheus
now natively supports OTLP, and considering that the OpenTelemetry Rust project
is still working towards a 1.0 release, we need to focus on essential components
to maintain scope and ensure timely delivery.

Prometheus interoperability remains a key goal for OpenTelemetry. However, the
current `opentelemetry-prometheus` crate requires a major rewrite to eliminate
dependencies on unmaintained crates. We may reintroduce a dedicated Prometheus
exporter in the future once these issues are resolved.

### Migration Guide

For those using Prometheus as a backend, you can integrate with Prometheus using
the following methods:

1. Use the OTLP Exporter to push metrics directly to Prometheus.
2. If you require a pull (scrape) model, push metrics to an OpenTelemetry
   Collector using the OTLP Exporter, and configure Prometheus to scrape the
   OpenTelemetry Collector.

These alternatives ensure continued Prometheus integration while allowing us to
focus on achieving a stable 1.0 release for OpenTelemetry Rust.

## Next Release

In the [next
release](https://github.com/open-telemetry/opentelemetry-rust/milestone/21), we
expect to stabilize the Metrics SDK and resolve the long-standing question of
`tokio-tracing` vs. `opentelemetry tracing`, which is a prerequisite before
stabilizing Distributed Tracing. Additionally, `Context` is also expected to be
enhanced with the ability to suppress telemetry-induced-telemetry.

## Instrumentation Libraries

Unlike other OpenTelemetry language implementations, OpenTelemetry Rust historically did not
maintain any instrumentations directly. This has recently changed with a
[contribution](https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/202)
from one of the founding members of the OpenTelemetry Rust project to the
contrib repository, providing an instrumentation library for
[`actix-web`](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-instrumentation-actix-web).
We expect that this instrumentation will serve as a reference implementation demonstrating best practices for
creating OpenTelemetry instrumentations in Rust.

We welcome additional contributions of instrumentation libraries to the contrib repository.

## Thanks

Thank you to everyone who contributed to this milestone. Please share your feedback
through GitHub issues or join the discussion in the OTel-Rust Slack channel
[here](https://cloud-native.slack.com/archives/C03GDP0H023).
