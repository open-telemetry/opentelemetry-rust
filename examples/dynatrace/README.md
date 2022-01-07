# Dynatrace Example

This is an advanced example showing the ingestion of trace data and of metric data together.

[Dynatrace documentation for Rust]

## Overview

* [`opentelemetry-otlp`] - Used to ingest trace data
* [`opentelemetry-dynatrace`] - Used to ingest metric data

## Getting started

```sh
$ cargo build --manifest-path=examples/dynatrace/Cargo.toml
```

[Dynatrace documentation for Rust]: https://www.dynatrace.com/support/help/extend-dynatrace/opentelemetry/opentelemetry-ingest/opent-rust/
[`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
[`opentelemetry-dynatrace`]: https://crates.io/crates/opentelemetry-dynatrace
