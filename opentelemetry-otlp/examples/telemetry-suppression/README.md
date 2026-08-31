# OTLP telemetry suppression example

This example sends OpenTelemetry Logs over OTLP/HTTP using an async `reqwest`
client. Export work runs on a dedicated Tokio runtime whose threads suppress
OpenTelemetry, preventing the exporter's transport events from being exported
back through the same pipeline. Application telemetry remains unaffected.

The example intentionally leaves its formatting layer unfiltered. Transport
events can therefore still appear locally even though the OpenTelemetry layer
does not export them.

## Run the example

Start an OTLP collector listening for HTTP/protobuf on `localhost:4318`. The
[`basic-otlp-http` collector
configuration](../basic-otlp-http/otel-collector-config.yaml) can be used for
this.

From the repository root, run:

```shell
cargo run -p telemetry-suppression
```

The collector should receive the `my-application` log. Transport events shown
by the local formatting layer should not appear in the collector output.
