# OpenTelemetry Rust Guidance

Guidance documents for using OpenTelemetry in Rust applications.

- [logs.md](logs.md) — Logs and Events
- [traces.md](traces.md) — Distributed Traces
- [metrics.md](metrics.md) — Metrics

## Why this guidance exists

Rust had a mature observability ecosystem before OpenTelemetry Rust matured.
The [`tracing`] crate was created by the Tokio project for structured logging
and in-process context propagation in async Rust, with deep integration into
the async runtime. It was never designed around the OpenTelemetry data model.

OpenTelemetry came later with a different scope: a vendor-neutral standard
for **distributed** tracing — spans that cross process boundaries — adopting
[W3C Trace Context] for propagation, with first-class concepts like span
kind, links, and remote parents. The OpenTelemetry Tracing API in this repo
is built around that data model.

The third-party [`tracing-opentelemetry`] crate bridges `tracing` spans into
OpenTelemetry spans. It predates parts of OpenTelemetry's evolution, and
because `tracing` itself has no first-class notion of OpenTelemetry-specific
span concepts, the bridge cannot fully express the OpenTelemetry model on
its own.

These docs reflect that history: we recommend `tracing` for logs and events
(where it excels), and the OpenTelemetry Tracing API for spans (where it is
the spec-aligned choice). For users invested in `tracing::span!`, the
bridge remains a viable option, with the caveats noted in
[traces.md](traces.md).

## A note on guidance

This is guidance, not policy. Rust users have a strong, established
ecosystem and the freedom to combine these libraries in ways that fit their
applications. Where we make a firm recommendation, it reflects what we
believe gives the best alignment with the OpenTelemetry specification and
the broadest compatibility — but the choice remains yours.

[`tracing`]: https://crates.io/crates/tracing
[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[W3C Trace Context]: https://www.w3.org/TR/trace-context/
