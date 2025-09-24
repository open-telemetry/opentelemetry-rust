# Batch Span Processor Tokio deadlock reproduction

This crate demonstrates how the asynchronous BatchSpanProcessor from
`opentelemetry-sdk` deadlocks when `force_flush` is invoked from a Tokio
current-thread runtime.

## Steps

```bash
# run the example; it will print the first line and then hang
cargo run -p batch-span-processor-hang-repro
```

The program configures `BatchSpanProcessor::builder(..., runtime::Tokio)` and
then calls `SdkTracerProvider::force_flush()` while running inside a
`#[tokio::main(flavor = "current_thread")]` context. The processor calls
`futures_executor::block_on` internally, so the same Tokio thread that is blocked
is also responsible for driving the background worker, resulting in a deadlock.

To see the hang, you can run with a timeout:

```bash
# exits with status 124 because the process never completes
timeout 5 cargo run -p batch-span-processor-hang-repro
```
