# OpenTelemetry OTLP Transform

This crate provides experimental conversions from OpenTelemetry SDK telemetry
into generated OTLP protobuf export request types and, optionally, zPages trace
data. It contains no exporter or transport.

> **Warning:** This crate's public API may change before stabilisation.

The public API intentionally exposes one request conversion entry point per
signal:

- `trace::export_request`
- `metrics::export_request`
- `logs::export_request`

With the `zpages` feature, the `tracez` module also provides free-function
replacements for the former `opentelemetry-proto` tracez conversions:

- `tracez::span_to_latency_data`
- `tracez::span_to_error_data`
- `tracez::span_to_running_data`
- `tracez::event_to_span_event`

Trace and log conversions accept a `ResourceAttributesWithSchema` prepared from
an SDK `Resource`. Custom exporters should prepare it once and reuse it across
requests, matching the SDK exporter lifecycle:

```rust,ignore
let resource = opentelemetry_otlp_transform::ResourceAttributesWithSchema::from(&resource);
let request = opentelemetry_otlp_transform::trace::export_request(spans, &resource);
```

Internal grouping and field-level conversion helpers are not public API.
