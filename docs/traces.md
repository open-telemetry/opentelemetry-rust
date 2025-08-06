# OpenTelemetry Rust Traces

Status: **Work-In-Progress**

## Introduction

This document provides comprehensive guidance on leveraging OpenTelemetry traces
in Rust applications.

## Instrumentation Guidance

1. **Use OTel API for distributed traces**

   Use the `opentelemetry::trace` API to create spans. This supports context
   propagation, span kinds (server/client), links, and remote parents.

2. **Use tracing for logs/events**

   Use `tracing::info!`, `tracing::event!`, etc. for structured logging. This
   will be converted to OTel LogRecords via opentelemetry-appender-tracing and
   will be automatically correlated with the current active OTel trace context
   as well.

3. **In-proc contextual enrichment for logs/events**

   Use `tracing::span!` macros to add contextual metadata (e.g., filename) that
   applies to a group of logs. The `otel-appender-tracing` crate will be
   enhanced to extract span attributes and attach them to logs automatically.

   OpenTelemetry does not have a spec-ed out solution for in-process contextual
   enrichment. This is very specific to the logging library (tracing) and its
   bridge.

4. **If using tracing::span! to create spans**

   This is not directly supported by OpenTelemetry. Use the
   `tracing-opentelemetry` bridge to convert tracing spans into OTel spans.

   There are some limitations with this approach arising due to `tracing`s lack of support for
   creating Spans following OpenTelemetry specification. Examples include
   - Cannot set remote parent
   - Cannot specify span kind (e.g., server/client)
   - Cannot add span links

   The bridge offers extension APIs to support some of these, but they are not
   standard and are maintained outside the OpenTelemetry and Tracing project and
   within the bridge itself.

   TODO: Should we make a recommendation about
   avoiding this extension APIs for instrumentation?

   If you are creating spans to track in-proc work (what OTel calls "internal" spans),
   `tracing:span` API is sufficient with `tracing-opentelemetry` bridge converting the
   `tracing` Span to OTel Span, and properly activating/de-activating OTel's context,
   to ensure correlation.

5. **Use instrumentation libraries when possible**

   If you're manually creating `tracing::span!` and converting to OTel span for
   "edge" spans, consider using official instrumentation libraries where
   available. These handle proper span creation and context propagation using
   the OpenTelemetry API directly.
