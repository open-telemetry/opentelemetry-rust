# OpenTelemetry Rust Traces

Status: **Work-In-Progress**

## Introduction

This document provides guidance on leveraging OpenTelemetry traces in Rust
applications.

In short: prefer instrumentation libraries that use the OpenTelemetry Tracing
API for framework-level spans, and use the OpenTelemetry Tracing API
directly to create your own custom spans. [`tracing`] is for logs and events,
not spans. See [logs.md](logs.md) for log guidance.

## Instrumentation Guidance

1. **Prefer instrumentation libraries.** For framework-level spans (HTTP
   servers/clients, database drivers, messaging), prefer instrumentation
   libraries that use the OpenTelemetry Tracing API directly. Most
   applications start by plugging these in to capture the
   request/response/downstream shape, then add custom spans for in-process
   work as needed. No stable instrumentation libraries exist yet in the
   OpenTelemetry Rust ecosystem, but in-progress ones for Tower and
   Actix-Web exist in the [opentelemetry-rust-contrib] repository:
   [`opentelemetry-instrumentation-tower`] and
   [`opentelemetry-instrumentation-actix-web`].

2. **Use the OpenTelemetry Tracing API to create custom spans.** The
   `opentelemetry::trace` API is designed around the OpenTelemetry
   specification, with first-class support for span kind
   (server/client/producer/consumer/internal), links, remote parents, and
   context propagation across process boundaries.

3. **[`tracing`] is designed to collect structured, event-based diagnostic
   information, and is not a complete substitute for the OpenTelemetry
   Tracing API, which focuses on distributed tracing.** The `tracing` crate
   does not have a first-class notion of an OpenTelemetry Span. It cannot,
   on its own, set span kind, attach links, or set a remote parent —
   concepts central to the OpenTelemetry specification, particularly for
   *edge* spans (see #4 below for nuance). Use it primarily for logs and
   events (see [logs.md](logs.md)).

4. **Bridging from `tracing::span!` to OpenTelemetry spans.** If you are
   already using `tracing::span!` and want those spans surfaced as
   OpenTelemetry spans, the third-party [`tracing-opentelemetry`] crate
   provides a bridge. It is maintained outside the OpenTelemetry project
   and is not part of this repo; we mention it here for completeness.

   For *internal* spans (spans that represent in-process work and never cross
   a process boundary), `tracing::span!` through this bridge produces a
   result nearly identical to using the OpenTelemetry Tracing API directly —
   span kind, links, and remote parent are not relevant for internal spans.
   The `tracing` limitations matter primarily for *edge* spans (e.g.,
   incoming/outgoing HTTP, messaging), where span kind, links, and remote
   parents are central to the OpenTelemetry data model. The bridge offers
   extension APIs to express these concepts.

## See Also

- [OpenTelemetry Traces Specification](https://opentelemetry.io/docs/specs/otel/trace/)
- [Main README](../README.md)
- [logs.md](logs.md) — guidance for logs/events
- [examples/tracing-http-propagator](../examples/tracing-http-propagator/) — end-to-end span creation and W3C context propagation
- [examples/tracing-grpc](../examples/tracing-grpc/) — span creation and propagation over gRPC

## TODO

This document is intentionally high-level. Areas to expand over time, similar
to the depth in [metrics.md](metrics.md):

- Best practices, with links to runnable examples
- `TracerProvider` lifecycle and shutdown
- Sampling strategies and configuration
- Context propagation: W3C Trace Context, Baggage, custom propagators
- Span attribute modelling and semantic conventions
- Performance considerations (allocation, attribute cost, span overhead)
- Common pitfalls (broken context, missed parents, mis-set span kind)
- Batching, exporter configuration, and back-pressure

[`tracing`]: https://crates.io/crates/tracing
[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[opentelemetry-rust-contrib]: https://github.com/open-telemetry/opentelemetry-rust-contrib
[`opentelemetry-instrumentation-tower`]: https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-instrumentation-tower
[`opentelemetry-instrumentation-actix-web`]: https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-instrumentation-actix-web
