# Environment Variable Propagation Example

This example demonstrates distributed tracing across a parent process and a
child process using environment variables as the propagation carrier.

## What This Example Does

1. The parent process configures the W3C `TraceContextPropagator` and starts a
   root span.
2. The parent copies its environment into an `EnvVarInjector`, injects that span
   context into the copy, then passes the full environment copy to a child
   process.
3. The child process wraps its startup environment entries with
   `EnvVarExtractor::from_os_entries(std::env::vars_os())`, extracts the
   propagated context, and starts its own span as a child of the parent's
   span.

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
