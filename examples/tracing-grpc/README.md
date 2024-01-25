# GRPC example

Example showing [Tonic] client and server interaction with OpenTelemetry context
propagation. Traces are exported to stdout.

[Tonic]: https://github.com/hyperium/tonic

## Running the example

```shell
# Run the server first
$ cargo run --bin grpc-server

# Now run the client to make a request to the server
$ cargo run --bin grpc-client
```

Observe that the traces are exported to stdout, and that they share the same
TraceId. Also, the server span would be parented to the client span. The example
demonstrates how to propagate OpenTelemetry context when making out-of-process
calls, so as to ensure the same trace is continued in the next process.

