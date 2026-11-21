# Getting started with OpenTelemetry Rust Tracing - gRPC Example

This example demonstrates the basics of distributed tracing with OpenTelemetry
in Rust, using a [Tonic](https://github.com/hyperium/tonic) gRPC client and
server. While not the absolute simplest tracing example, it shows a realistic,
end-to-end scenario: a single trace that spans two processes.

## Understanding OpenTelemetry Tracing

OpenTelemetry provides an end-user facing Tracing API. Application and library
authors use this API directly (via the
[`opentelemetry`](https://docs.rs/opentelemetry/latest/opentelemetry/) crate)
to create **spans** that represent units of work. The
[`opentelemetry-sdk`](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/)
crate provides the implementation that processes those spans and forwards them
to one or more exporters.

A few key concepts:

- **Span** - a single, named, timed operation (e.g. "handle HTTP request",
  "query database"). Spans can carry attributes and events.
- **Trace** - a tree of related spans, sharing a single `TraceId`, that
  together represent the lifecycle of one logical request or operation.
- **Context Propagation** - the mechanism that carries trace identifiers
  across process boundaries (e.g. via HTTP headers or gRPC metadata) so that
  spans created in different processes are stitched together into one trace.

## What This Example Does

This example consists of two binaries - a gRPC client and a gRPC server -
sharing a simple "Greeter" service definition:

1. Both client and server set up an OpenTelemetry `SdkTracerProvider` with a
   **stdout exporter** (for simplicity) and register the W3C
   `TraceContextPropagator` as the global propagator. This propagator defines
   *how* trace context is serialized into and deserialized from carrier
   formats (HTTP headers, gRPC metadata, etc.) using the
   [W3C Trace Context](https://www.w3.org/TR/trace-context/) standard - it is
   the piece that makes steps 2 and 3 below possible.
2. The **client** creates a root span (`Greeter/client`) for the outgoing
   call, then uses the globally-registered propagator to **inject** the
   current trace context into the gRPC request metadata before sending it.
3. The **server** uses the same globally-registered propagator to **extract**
   the trace context from the incoming gRPC metadata and creates its own
   span (`Greeter/server`) as a child of the client's span.
4. Both spans are exported to stdout. Because they share the same `TraceId`
   and the server span references the client span as its parent, the two
   pieces are correlated into a single distributed trace.

Without the propagator setup in step 1, the inject/extract calls would have
nothing to do and the server would create an unrelated root span instead of a
child of the client - giving you two disconnected traces rather than one.

**Note on Exporters**: This example uses the stdout exporter for demonstration
purposes. In production scenarios, you would typically use other exporters such
as:

- **OTLP exporter** (`opentelemetry-otlp`) to send traces to an OpenTelemetry
  Collector or compatible backend. See the [OTLP
  example](../../opentelemetry-otlp/examples/basic-otlp/README.md) for details.
- Other vendor-specific exporters for your observability platform.

## Usage

```shell
# Run the server first
$ cargo run --bin grpc-server

# Now run the client to make a request to the server
$ cargo run --bin grpc-client
```

Observe in the stdout output that both spans share the same `TraceId`, and
that the server span is parented to the client span - confirming that the
trace was successfully propagated across the process boundary.
