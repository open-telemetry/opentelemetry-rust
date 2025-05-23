# Release Notes 0.30

OpenTelemetry Rust 0.30 introduces a few breaking changes to the
`opentelemetry_sdk` crate in the `metrics` feature. These changes were essential
to drive the Metrics SDK towards stability. With this release, the Metrics SDK
is officially declared stable. The Metrics API was declared stable last year,
and previously, the Logs API, SDK, and OTel-Appender-Tracing were also marked
stable. Importantly, no breaking changes have been introduced to components
already marked as stable.

It is worth noting that the `opentelemetry-otlp` crate remains in a
Release-Candidate state and is not yet considered stable. With the API and SDK
for Logs and Metrics now stable, the focus will shift towards further refining
and stabilizing the OTLP Exporters in upcoming releases. Additionally,
Distributed Tracing is expected to progress towards stability, addressing key
interoperability challenges.

For detailed changelogs of individual crates, please refer to their respective
changelog files. This document serves as a summary of the main changes.

## Key Changes

### Metrics SDK Improvements

1. **Stabilized "view" features**: Previously under an experimental feature
   flag, views can now be used to modify the name, unit, description, and
   cardinality limit of a metric. Advanced view capabilities, such as changing
   aggregation or dropping attributes, remain under the experimental feature
   flag.

2. **Cardinality capping**: Introduced the ability to cap cardinality and
   configure limits using views.

3. **Polished public API**: Refined the public API to hide implementation
   details from exporters, enabling future internal optimizations and ensuring
   consistency. Some APIs related to authoring custom metric readers have been
   moved behind experimental feature flags. These advanced use cases require
   more time to finalize the API surface before being included in the stable
   release.

### Context-Based Suppression

Added the ability to suppress telemetry based on Context. This feature prevents
telemetry-induced-telemetry scenarios and addresses a long-standing issue. Note
that suppression relies on proper context propagation. Certain libraries used in
OTLP Exporters utilize `tracing` but do not adopt OpenTelemetry's context
propagation. As a result, not all telemetry is automatically suppressed with
this feature. Improvements in this area are expected in future releases.

## Next Release

In the [next
release](https://github.com/open-telemetry/opentelemetry-rust/milestone/22), the
focus will shift to OTLP Exporters and Distributed Tracing, specifically
resolving
[interoperability](https://github.com/open-telemetry/opentelemetry-rust/issues/2420)
issues with `tokio-tracing` and other fixes required to drive Distributed
Tracing towards stability.

## Acknowledgments

Thank you to everyone who contributed to this milestone. We welcome your
feedback through GitHub issues or discussions in the OTel-Rust Slack channel
[here](https://cloud-native.slack.com/archives/C03GDP0H023).

We are also excited to announce that [Anton Grübel](https://github.com/gruebel)
and [Björn Antonsson](https://github.com/bantonsson) have joined the OTel Rust
project as Approvers.
