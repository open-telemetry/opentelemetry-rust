# OpenTelemetry B3 Propagator

B3 trace context propagation for OpenTelemetry Rust.
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
with a valid single header taking precedence. `Propagator::new()` injects the
single `b3` header, as required by the OpenTelemetry specification. Use
`Propagator::with_encoding(B3Encoding::MultipleHeader)` to inject the multiple
`x-b3-*` headers instead.

## Migrating from opentelemetry-zipkin

Replace your `opentelemetry-zipkin` dependency with `opentelemetry-propagator-b3`
if you only use B3 propagation, and change the import:

```rust
use opentelemetry_propagator_b3::{B3Encoding, Propagator};
```

The new crate preserves the existing API. Note that `Propagator::new()` now
injects the single `b3` header, while the `opentelemetry-zipkin` propagator
injected the multiple `x-b3-*` headers by default. Use
`Propagator::with_encoding(B3Encoding::MultipleHeader)` to keep that behavior.

## Supported Rust Versions

This crate requires Rust 1.75 or later.

## License

Apache License 2.0.
