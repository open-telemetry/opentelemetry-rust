# OpenTelemetry-Rust

[![Crates.io: opentelemetry](https://img.shields.io/crates/v/opentelemetry.svg)](https://crates.io/crates/opentelemetry)
[![Documentation](https://docs.rs/opentelemetry/badge.svg)](https://docs.rs/opentelemetry)
[![Crates.io](https://img.shields.io/crates/l/opentelemetry)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amaster)
[![Gitter chat](https://img.shields.io/badge/gitter-join%20chat%20%E2%86%92-brightgreen.svg)](https://gitter.im/open-telemetry/opentelemetry-rust)

The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.

OpenTelemetry provides a single set of APIs, libraries, agents, and collector
services to capture distributed traces and metrics from your application. You
can analyze them using [Prometheus], [Jaeger], and other observability tools.

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io

## Getting Started

```rust
use opentelemetry::{api::trace::Tracer, exporter::trace::stdout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instrumentation pipeline
    let (tracer, _uninstall) = stdout::new_pipeline().install();

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    Ok(())
}
```

See the [examples](./examples) directory for different integration patterns.

## Contributing

See the [contributing file](CONTRIBUTING.md).
