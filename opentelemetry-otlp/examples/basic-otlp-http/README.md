* The application send data directly to a Collector (port 4318)
* Run the application locally, to run as a docker container you have to change the relative paths from the `Cargo.toml`
* The Collector then sends the data to the appropriate backend, in this case Debug Exporter. The Debug Exporter exports data to console.

This demo uses `docker-compose` and by default runs against the `otel/opentelemetry-collector-dev:latest` image,
and uses `http` as the transport.

```shell
docker-compose up
```

In another terminal run the application `cargo run`

The docker-compose terminal will display logs, metrics and traces.

Tear it down:

```shell
docker-compose down
```


