# OTLP - Integration Tests

This directory contains integration tests for `opentelemetry-otlp`. It uses
[testcontainers](https://testcontainers.com/) to start an instance of the OTEL
collector using [otel-collector-config.yaml](otel-collector-config.yaml), which
then uses a file exporter per signal to write the output it receives back to the
host machine.

The tests connect directly to the collector on `localhost:4317` and
`localhost:4318`, push data through, and then check that what they expect has
popped back out into the files output by the collector.

## Pre-requisites

* Docker, for the test container
* TCP/4317 and TCP/4318 free on your local machine. If you are running another
  collector, you'll need to stop it for the tests to run.

## Test Matrix

| **Exporter Type**   | **Provider Creation**     | **Logging**           | **Network Clients**        | **Status** |
|----------------------|---------------------------|------------------------|----------------------------|------------|
| **Simple**           | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `tonic`                    | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `reqwest`                  | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `reqwest-blocking`         | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `hyper`                    | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `tonic`                   | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `reqwest`                 | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `reqwest-blocking`        | Works      |
| **Simple**           | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `hyper`                   | Works      |
| **Simple**           | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `tonic`                   | Works      |
| **Simple**           | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `reqwest`                 | Works      |
| **Simple**           | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `reqwest-blocking`        | Works      |
| **Simple**           | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `hyper`                   | Works      |
| **Simple**           | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `tonic`                  | Works      |
| **Simple**           | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `reqwest`                | Works      |
| **Simple**           | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `reqwest-blocking`       | Works      |
| **Simple**           | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `hyper`                  | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `tonic`                   | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `reqwest`                 | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `reqwest-blocking`        | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Inside `tokio::Runtime` | `hyper`                   | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `tonic`                  | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `reqwest`                | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `reqwest-blocking`       | Works      |
| **Batch**            | Inside `tokio::Runtime`  | Outside `tokio::Runtime` | `hyper`                  | Works      |
| **Batch**            | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `tonic`                  | Works      |
| **Batch**            | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `reqwest`                | Works      |
| **Batch**            | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `reqwest-blocking`       | Works      |
| **Batch**            | Outside `tokio::Runtime` | Inside `tokio::Runtime` | `hyper`                  | Works      |
| **Batch**            | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `tonic`                 | Works      |
| **Batch**            | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `reqwest`               | Works      |
| **Batch**            | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `reqwest-blocking`      | Works      |
| **Batch**            | Outside `tokio::Runtime` | Outside `tokio::Runtime` | `hyper`                 | Works      |


