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

## aws-xray
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-aws
- opentelemetry-http

The application is built using `hyper`.

Check this example if you want to understand *how to use opentelemetry with AWS X-Ray*.

## basic
**Tracing, Metrics**

This example uses following crates from this repo:
- opentelemetry(tracing, metrics)
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to send spans to Jaeger and display metrics in stdout from a tokio 
application*.

## basic-otlp
**Tracing, Metrics**

This example uses following crates from this repo:
- opentelemetry(tracing, metrics)
- opentelemetry-otlp

The application is built using `tokio`.

Check this example if you want to understand *how to send spans and metrics to an opentelemetry collector from a tokio 
application using Grpc*.

## basic-otlp-http
**Tracing, Metrics**

This example uses following crates from this repo:
- opentelemetry(tracing, metrics)
- opentelemetry-otlp

The application is built using `tokio`.

Check this example if you want to understand *how to send spans and metrics to an opentelemetry collector from a tokio 
application using HTTP*.

## basic-otlp-with-selector
**Tracing, Metrics**

This example uses following crates from this repo:
- opentelemetry(tracing, metrics)
- opentelemetry-otlp

The application is built using `tokio`.

Check this example if you want to understand *how to use custom aggregation selector and custom export kind in metrics*.

## datadog
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-datadog

Check this example if you want to understand *how to send spans to a datadog collector*.

## external-otlp-grpcio-async-std
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-otlp

The application is built using `async-std` and `grpcio`.

Check this example if you want to understand *how to send spans to an OTLP compatible collector from an async-std 
application*.

## external-otlp-tonic-tokio
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-otlp

The application is built using `tokio` and `tonic`.

Check this example if you want to understand *how to send spans to an OTLP compatible collector from a tokio application 
using TLS*.

## grpc
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `tokio` and `tonic`.

Check this example if you want to understand *how to propagate context with tonic(Grpc)*.

## http
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-http

The application is built using `hyper`.

Check this example if you want to understand *how to propagate context with HTTP and tracing between http client and http 
server, the spans will output to stdout*.

## hyper-prometheus
**Metrics**

This example uses following crates from this repo:
- opentelemetry(metrics)
- opentelemetry-prometheus

The application is built using `hyper`.

Check this example if you want to understand *how to send metrics to prometheus in opentelemetry*.

## multiple-span-processors
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-zipkin
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to set up multiple span processors within an application*.

## tracing-grpc
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-jaeger

The application is built using `tokio`.

Check this example if you want to understand *how to integrate tracing with opentelemetry*.

## zipkin
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-zipkin

Check this example if you want to understand *how to send spans to zipkin*.

## zpages
**Tracing**

This example uses following crates from this repo:
- opentelemetry(tracing)
- opentelemetry-zpages

The application is built using `tokio` and `hyper`.

Check this example if you want to understand *how to set up a zpage server to debug tracing issues*.
