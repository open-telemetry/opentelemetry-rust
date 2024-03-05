# Examples

This directory contains some examples that should help you get start crates from `opentelemetry-rust`.

## log-basic

This example uses following crates from this repo:

- opentelemetry(log)
- opentelemetry-appender-log
- opentelemetry-stdout

Check this example if you want to understand *how to instrument logs using opentelemetry*.

## metrics-basic

This example uses following crates from this repo:

- opentelemetry(metrics)
- opentelemetry-stdout

Check this example if you want to understand *how to instrument metrics using opentelemetry*.

## metrics-advanced

This example uses following crates from this repo:

- opentelemetry(metrics)
- opentelemetry-stdout

This builds on top of the metrics-basic,
and shows advanced features in Metrics SDK like using Views.

## tracing-grpc

This example uses following crates from this repo:

- opentelemetry(tracing)
- opentelemetry-stdout

The application is built using `tokio`.

Check this example if you want to understand *how to create spans and
propagate/restore context in OpenTelemetry* in a gRPC client-server application.

## tracing-http-propagator

This example uses following crates from this repo:

- opentelemetry(tracing)
- opentelemetry-http
- opentelemetry-stdout

Check this example if you want to understand *how to create spans and
propagate/restore context in OpenTelemetry* in an HTTP client-server
application.

## tracing-jaeger

This example uses following crates from this repo:

- opentelemetry(tracing)
- opentelemetry-otlp

The application is built using `tokio`.

Check this example if you want to understand *how to use OTLP Exporter to export traces to Jaeger*.
