# Basic OTLP exporter Example

This example shows how to setup OpenTelemetry OTLP exporter for logs, metrics
and traces to export them to the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP
over selected protocol such as HTTP/protobuf or HTTP/json. The Collector then sends the data to the appropriate
backend, in this case, the logging Exporter, which displays data to console.

## Usage

### `docker-compose`

By default runs against the `otel/opentelemetry-collector:latest` image, and uses `reqwest-client`
as the http client, using http as the transport.

```shell
docker-compose up
```

In another terminal run the application `cargo run`

The docker-compose terminal will display logs, traces, metrics.

Press Ctrl+C to stop the collector, and then tear it down:

```shell
docker-compose down
```

### Manual

If you don't want to use `docker-compose`, you can manually run the `otel/opentelemetry-collector` container
and inspect the logs to see traces being transferred.

On Unix based systems use:

```shell
# From the current directory, run `opentelemetry-collector`
docker run --rm -it -p 4318:4318 -v $(pwd):/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

On Windows use:

```shell
# From the current directory, run `opentelemetry-collector`
docker run --rm -it -p 4318:4318 -v "%cd%":/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

Run the app which exports logs, metrics and traces via OTLP to the collector

```shell
cargo run
```


By default the app will use a `reqwest` client to send. A hyper 0.14 client can be used with the `hyper` feature enabled

```shell
cargo run --no-default-features --features=hyper
```


## View results

You should be able to see something similar below with different time and ID in the same console that docker runs.

### Span

```text
...
2024-05-14T02:15:56.827Z        info    ResourceSpans #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeSpans #0
ScopeSpans SchemaURL:
InstrumentationScope basic
InstrumentationScope attributes:
     -> scope-key: Str(scope-value)
Span #0
    Trace ID       : 4467894e2d8d0c4165df1218160bc260
    Parent ID      : 589ea953b6ec03a9
    ID             : b2aa3c3a9c21e0d0
    Name           : Sub operation...
    Kind           : Internal
    Start time     : 2024-05-14 02:15:56.824239163 +0000 UTC
    End time       : 2024-05-14 02:15:56.824244315 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> another.key: Str(yes)
Events:
SpanEvent #0
     -> Name: Sub span event
     -> Timestamp: 2024-05-14 02:15:56.82424188 +0000 UTC
     -> DroppedAttributesCount: 0
ResourceSpans #1
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeSpans #0
ScopeSpans SchemaURL:
InstrumentationScope basic
InstrumentationScope attributes:
     -> scope-key: Str(scope-value)
Span #0
    Trace ID       : 4467894e2d8d0c4165df1218160bc260
    Parent ID      :
    ID             : 589ea953b6ec03a9
    Name           : Main operation
    Kind           : Internal
    Start time     : 2024-05-14 02:15:56.824194899 +0000 UTC
    End time       : 2024-05-14 02:15:56.824251136 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> another.key: Str(yes)
Events:
SpanEvent #0
     -> Name: Nice operation!
     -> Timestamp: 2024-05-14 02:15:56.824201397 +0000 UTC
     -> DroppedAttributesCount: 0
     -> Attributes::
          -> bogons: Int(100)
        {"kind": "exporter", "data_type": "traces", "name": "logging"}
...
```

### Metric

```text
...
2024-05-14T02:15:56.827Z        info    ResourceMetrics #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeMetrics #0
ScopeMetrics SchemaURL: schema_url
InstrumentationScope basic v1.0
InstrumentationScope attributes:
     -> scope-key: Str(scope-value)
Metric #0
Descriptor:
     -> Name: test_counter
     -> Description: a simple counter for demo purposes.
     -> Unit: my_unit
     -> DataType: Sum
     -> IsMonotonic: true
     -> AggregationTemporality: Cumulative
NumberDataPoints #0
Data point attributes:
     -> test_key: Str(test_value)
StartTimestamp: 2024-05-14 02:15:56.824127393 +0000 UTC
Timestamp: 2024-05-14 02:15:56.825354918 +0000 UTC
Value: 11
...
```

### Logs

```text
...
2024-05-14T02:15:56.828Z        info    ResourceLog #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-log-appender 0.3.0
LogRecord #0
ObservedTimestamp: 2024-05-14 02:15:56.824218088 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from banana. My price is 2.99. I am also inside a Span!)
Trace ID: 4467894e2d8d0c4165df1218160bc260
Span ID: 589ea953b6ec03a9
Flags: 1
ResourceLog #1
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-log-appender 0.3.0
LogRecord #0
ObservedTimestamp: 2024-05-14 02:15:56.824254268 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from apple. My price is 1.99)
Trace ID:
Span ID:
Flags: 0
...
```
