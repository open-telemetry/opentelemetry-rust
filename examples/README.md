# Examples
This folder contains some examples that should help you get start crates from `opentelemetry-rust`.

## log-basic
**Logs**

This example uses following crates from this repo:
- opentelemetry(log)
- opentelemetry-appender-log
- opentelemetry-stdout

Check this example if you want to understand *how to instrument logs using opentelemetry*.

## metrics-basic
**Metrics**

This example uses following crates from this repo:
- opentelemetry(metrics)
- opentelemetry-stdout

Check this example if you want to understand *how to instrument metrics using opentelemetry*.

## traceresponse
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-http
- opentelemetry-contrib(TraceContextResponsePropagator)
- opentelemetry-stdout

## tracing-grpc
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to integrate tracing with opentelemetry*.