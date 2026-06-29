# Environment Variable Propagation Example

This example demonstrates distributed tracing across a parent process and a
child process using environment variables as the propagation carrier.

## What This Example Does

1. The parent process configures the W3C `TraceContextPropagator` and starts a
   root span.
2. The parent injects that span context into an `EnvVarInjector`, then passes
   the resulting environment variables to a child process. The child inherits
   the rest of the parent's environment through `std::process::Command`.
3. The child process builds an `EnvVarExtractor` from the active propagator's
   fields, extracts the propagated context, and starts its own span as a child
   of the parent's span.

The example prints the parent's `TraceId` and `SpanId`, then prints the child's
`TraceId` and remote parent `SpanId` so you can see that the trace was
propagated across the process boundary.

## Usage

From `examples/env-var-propagation`, run:

```shell
cargo run
```

The output should show:

- the parent and child spans sharing the same `TraceId`
- the child's reported `parent_span_id` matching the parent's `span_id`

Both processes also export their spans to stdout using the OpenTelemetry stdout
exporter.
