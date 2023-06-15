# GRPC example

Example showing [Tonic] client and server interaction with OpenTelemetry context propagation.  [tracing_opentelemetry](https://docs.rs/tracing-opentelemetry/0.4.0/tracing_opentelemetry/) is used to hook into the [tracing](https://github.com/tokio-rs/tracing) ecosystem, which enables drop-in replacements for [log](https://github.com/rust-lang/log) macros and an `#[instrument]` macro that will automatically add spans to your functions.

[Tonic]: https://github.com/hyperium/tonic

Examples
--------

```shell
# Run jaeger in background
$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 jaegertracing/all-in-one:latest

# Run the server
$ cargo run --bin grpc-server

# Now run the client to make a request to the server
$ cargo run --bin grpc-client

# View spans (see the image below)
$ firefox http://localhost:16686/
```

![Jaeger UI](trace.png)
