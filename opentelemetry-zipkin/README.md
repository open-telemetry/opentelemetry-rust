# OpenTelemetry Zipkin exporter (removed)

The `opentelemetry-zipkin` crate has been removed from this repository after being
deprecated in 0.32.0. Version 0.33.0 is the final release and remains available on
[crates.io](https://crates.io/crates/opentelemetry-zipkin/0.33.0).

## Exporting traces

Use [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp) to export traces
via OTLP. To continue using Zipkin as your backend, send OTLP to an OpenTelemetry
Collector configured with the
[Zipkin exporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/zipkinexporter),
or use a Zipkin deployment with the
[zipkin-otel server module](https://github.com/openzipkin-contrib/zipkin-otel).

## B3 propagation

Use [`opentelemetry-propagator-b3`](https://crates.io/crates/opentelemetry-propagator-b3).
B3 propagation remains supported. Replace the `opentelemetry-zipkin` dependency with
`opentelemetry-propagator-b3` and update your imports:

```rust
use opentelemetry_propagator_b3::{B3Encoding, Propagator};
```

See the [B3 migration guide](../opentelemetry-propagator-b3/README.md#migrating-from-opentelemetry-zipkin).

## Previous releases

The [changelog](CHANGELOG.md) and
[0.33.0 documentation](https://docs.rs/opentelemetry-zipkin/0.33.0/opentelemetry_zipkin/)
remain available for existing users.
