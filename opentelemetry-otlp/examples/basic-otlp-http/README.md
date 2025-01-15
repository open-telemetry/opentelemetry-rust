# Basic OTLP Exporter Example

This example demonstrates how to set up an OpenTelemetry OTLP exporter for logs,
metrics, and traces to send data to the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP
over HTTP (using `protobuf` encoding by default but can be changed to use
`json`). The Collector then forwards the data to the configured backend, which
in this case is the logging exporter, displaying data on the console.
Additionally, the example configures a `tracing::fmt` layer to output logs
emitted via `tracing` to `stdout`. For demonstration, this layer uses a filter
to display `DEBUG` level logs from various OpenTelemetry components. In real
applications, these filters should be adjusted appropriately.

The example employs a `BatchExporter` for logs and traces, which is the
recommended approach when using OTLP exporters. While it can be modified to use
a `SimpleExporter`, this requires making the main function a regular main and
*not* tokio main.

// TODO: Document how to use hyper client.

## Usage

Run the `otel/opentelemetry-collector` container using docker
and inspect the logs to see the exported telemetry.

On Unix based systems use:

```shell
# From the current directory, run `opentelemetry-collector`
docker run --rm -it -p 4317:4317 -p 4318:4318 -v $(pwd):/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

On Windows use:

```shell
# From the current directory, run `opentelemetry-collector`
docker run --rm -it -p 4317:4317 -p 4318:4318 -v "%cd%":/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

Run the app which exports logs, metrics and traces via OTLP to the collector

```shell
cargo run
```

The app will use a `reqwest-blocking` client to send.

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
          -> some.key: Int(100)
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
