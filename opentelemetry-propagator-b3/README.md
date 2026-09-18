# OpenTelemetry B3 Propagator

B3 trace context propagation for OpenTelemetry Rust, independent of the telemetry exporter.
B3 remains supported even though the Zipkin exporter is deprecated. The
[OpenTelemetry specification](https://opentelemetry.io/docs/specs/otel/context/api-propagators/#propagators-distribution)
requires B3 to be distributed as a core package.

## Usage

```rust
use opentelemetry::global;
use opentelemetry_propagator_b3::Propagator;

global::set_text_map_propagator(Propagator::new());
```

The propagator accepts both single and multiple B3 headers when extracting context,
with a valid single header taking precedence. `Propagator::new()` retains the
multiple-header injection default of the implementation in `opentelemetry-zipkin`.
`Propagator::with_encoding` and `B3Encoding` provide the existing encoding options.

## Migrating from opentelemetry-zipkin

Replace your `opentelemetry-zipkin` dependency with `opentelemetry-propagator-b3`
if you only use B3 propagation, and change the import:

```rust
use opentelemetry_propagator_b3::{B3Encoding, Propagator};
```

The public types and propagation behavior are unchanged. The deprecated
`opentelemetry-zipkin` crate re-exports these types for compatibility.

## Supported Rust Versions

This crate requires Rust 1.75 or later.

## License

Apache License 2.0.
