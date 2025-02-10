# OpenTelemetry Lock Contention for Metrics - Example

This example demonstrates the performance difference of using a shared instance of `MeterProvider` in multiple threads vs
having a dedicated instance of `MeterProvider` for each thread.

## Usage

To run the example using a shared `MeterProvider`:

```shell
cargo run --release -- shared
```

To run the example using a per-thread `MeterProvider`:

```shell
cargo run --release -- per-thread
```

