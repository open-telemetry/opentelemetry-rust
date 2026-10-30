# OpenTelemetry OTLP Transform

This crate provides stable conversions from OpenTelemetry SDK telemetry into
generated OTLP protobuf export request types. It contains no exporter or
transport.

The public API intentionally exposes one conversion entry point per signal:

- `trace::export_request`
- `metrics::export_request`
- `logs::export_request`

Internal grouping and field-level conversion helpers are not public API.

The public conversion API follows the stability guarantees of the stable
OpenTelemetry Rust crates.
