# OTLP - Integration Tests

This directory contains integration tests for `opentelemetry-otlp`. It uses
[testcontainers](https://testcontainers.com/) to start an instance of the OTEL
collector using [otel-collector-config.yaml](otel-collector-config.yaml), which
then uses a file exporter per signal to write the output it receives back to the
host machine.

The tests connect directly to the collector on `localhost:4317` and
`localhost:4318`, push data through, and then check that what they expect has
popped back out into the files output by the collector.

Retry tests use an in-process fake OTLP endpoint instead of the collector. The
endpoint returns scripted failures so the tests can verify exporter retry
behavior through a real HTTP client.

The metric producer test also uses a local HTTP endpoint, without Docker. It
checks the decoded OTLP payload for SDK metrics and externally constructed
gauges, sums, histograms, exponential histograms, and exemplars. Run both reader
variants from the workspace root:

```shell
cargo test -p integration_test_runner --no-default-features --features reqwest-blocking-client --test metric_producer
cargo test -p integration_test_runner --no-default-features --features reqwest-client --test metric_producer
```

## Pre-requisites

* Docker, for the test container
* TCP/4317 and TCP/4318 free on your local machine. If you are running another
  collector, you'll need to stop it for the tests to run.
