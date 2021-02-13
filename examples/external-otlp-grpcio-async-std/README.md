# External OTLP collector with grpcio and async-std with TLS

This example shows basic span, and exports to OTLP enabled collectors, like honeycomb, lightstep and other services.
Use this service in case you don't use `tokio`s runtime, for example with web frameworks like `tide` or any `async-std` library that
makes you use it as a runtime.
As these services all reside outside your own infrastructure, they require TLS for encryption to ensure your data safety.
With this example, you can export to any service that supports OTLP by using environment variables.
The following example exports data to Honeycomb:

```shell
cd examples/external-otlp-grpcio-async-std/
OTLP_GRPCIO_ENDPOINT=https://api.honeycomb.io:443 \
OTLP_GRPCIO_X_HONEYCOMB_TEAM=token \
OTLP_GRPCIO_X_HONEYCOMB_DATASET=dataset \
cargo run
```

The only required variable is `OTLP_GRPCIO_ENDPOINT` and any other variable that beggins with the prefix `OTLP_GRPCIO_` will be sent as headers
e.g.: `OTLP_GRPCIO_X_HONEYCOMB_TEAM` becomes `x-honeycomb-team` and `OTLP_GRPCIO_X_HONEYCOMB_DATASET` becomes `x-honeycomb-dataset`.
