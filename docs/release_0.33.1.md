# Release Notes 0.33.1

Released 2026-Oct-08

**The OTLP exporters for Logs and Metrics are now stable.** The release-candidate period that began with 0.33.0 on September 18 did not identify a need for further breaking exporter API changes.

The Logs and Metrics API and SDK remain stable. Distributed Tracing remains in beta, and features explicitly marked experimental remain experimental. No migration is required from 0.33.0 for the crates published in this release.

## Included crates

All maintained crates in this repository are released together at 0.33.1:

- `opentelemetry`
- `opentelemetry_sdk`
- `opentelemetry-http`
- `opentelemetry-proto`
- `opentelemetry-otlp`
- `opentelemetry-semantic-conventions`
- `opentelemetry-propagator-b3`
- `opentelemetry-appender-tracing`
- `opentelemetry-appender-log`
- `opentelemetry-stdout`
- `opentelemetry-prometheus`

## Fixes and improvements

- **OTLP/JSON interoperability:** encode integer metric and exemplar values as decimal strings, accept both strings and numbers when decoding, and place exemplar values directly on the exemplar object. Omitted default fields in exponential histograms, summary quantiles, and exemplars are now accepted.
- **Metrics:** reject NaN and infinite measurements before they corrupt aggregates. Metric export failures are logged at ERROR, and a one-time error diagnostic reports when a panic terminates the default periodic reader's worker thread. Observable callback panic behavior is documented; this does not add panic recovery.
- **Context:** avoid a panic when dropping a context value whose destructor accesses the current context.
- **Tracing:** accept optional whitespace in `TraceState`, treat invalid parent span contexts as absent, and add a configurable `ParentBasedSampler` while preserving the existing sampler API. Span-to-protobuf conversion avoids cloning every span.
- **Prometheus:** retain explicitly selected resource labels when scope labels are disabled.
- **Protobuf:** add an optional `file-descriptors` feature for accessing the generated descriptors.

See each crate's changelog for details. Crates without implementation changes are included to keep versions aligned.
