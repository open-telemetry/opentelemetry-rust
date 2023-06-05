# Examples
This folder contains some examples that should help you get start crates from `opentelemetry-rust`.

## actix-http
**Tracing** 

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `actix-web`. 

Check this example if you want to understand *how to send spans from an actix-web application to Jaeger via HTTP*.

## actix-http-tracing
**Tracing**, **Metrics**

This example uses following crates from this repo:
- opentelemetry(tracing, metrics)
- opentelemetry-jaeger
- opentelemetry-prometheus

The application is built using `actix-web`.

Check this example if you want to understand *how to export data to Jaeger and Prometheus from an
actix-web app instrumented using the tracing API and ecosystem*.

## actix-udp
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `actix-web`. 

Check this example if you want to understand *how to send spans from an actix-web application to Jaeger via UDP*.

## async
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to instrument spans in async runtime*.

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

## multiple-span-processors
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-zipkin
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to set up multiple span processors within an application*.

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