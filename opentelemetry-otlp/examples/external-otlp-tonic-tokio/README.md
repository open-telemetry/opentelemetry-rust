# External OTLP collector with tonic and tokio with TLS

This example shows basic span, and exports to OTLP enabled collectors, like honeycomb, lightstep and other services.
As these services all reside outside your own infrastructure, they require TLS for encryption to ensure your data safety.
With this example, you can export to any service that supports OTLP by using environment variables.
The following example exports data to Honeycomb:

```shell
OTLP_TONIC_ENDPOINT=https://api.honeycomb.io:443 \
OTLP_TONIC_X_HONEYCOMB_TEAM=token \
OTLP_TONIC_X_HONEYCOMB_DATASET=dataset \'
cargo run --bin external-otlp-tonic-tokio
```

The only required variable is `OTLP_TONIC_ENDPOINT` and any other variable that beggins with the prefix `OTLP_TONIC_` will be sent as headers
e.g.: `OTLP_TONIC_X_HONEYCOMB_TEAM` becomes `x-honeycomb-team` and `OTLP_TONIC_X_HONEYCOMB_DATASET` becomes `x-honeycomb-dataset`.
