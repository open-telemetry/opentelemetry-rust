# Actix-web - Jaeger example with HTTP collector and batch exporter 

This example shows how to export spans from an actix-web application and ship
them to Jaeger via OTLP/gRPC.It uses the batch exporter to avoid excessive
network roundtrips to Jaeger.

Note: Jaeger supports native OTLP ingestion from
[v1.35](https://medium.com/jaegertracing/introducing-native-support-for-opentelemetry-in-jaeger-eb661be8183c).
If you are using older version and cannot upgrade, use [these
instructions](https://github.com/open-telemetry/opentelemetry-rust/tree/v0.19.0/examples/actix-http/README.md).

## Usage

Launch the application:
```shell
# Run jaeger in background with OTLP ingestion enabled.
$ docker run -d -p16686:16686 -p4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest

# Start the actix-web server 
$ cargo run

# View spans
$ firefox http://localhost:16686/
```

Fire a request:
```bash
curl http://localhost:8088
```
