# Exporting traces to Jaeger

This example shows how to export spans to Jaeger agent using OTLP.

## Usage

Launch the application:

```shell

# Run jaeger in background
$ docker run -d -p16686:16686 -p4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest

# Run the app
$ cargo run

# View spans
$ firefox http://localhost:16686/
```
