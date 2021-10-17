* The application send data directly to a Collector (port 4318)
* Run the application locally, to run as a docker container you have to change the relative paths from the `Cargo.toml`
* The Collector then sends the data to the appropriate backend, in this case JAEGER

This demo uses `docker-compose` and by default runs against the `otel/opentelemetry-collector-dev:latest` image.

```shell
docker-compose up
or
docker-compose up -d
```

In another terminal run the application `cargo run`

Use the browser to see the trace:
- Jaeger at http://0.0.0.0:16686

Tear it down:

```shell
docker-compose down
```


