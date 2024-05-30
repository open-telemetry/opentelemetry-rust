# Basic OTLP exporter Example

This example shows how to setup OpenTelemetry OTLP exporter for logs, metrics
and traces to exports them to the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP over gRPC.
The Collector then sends the data to the appropriate backend, in this case,
the logging Exporter, which displays data to console.

## Usage

### `docker-compose`

By default runs against the `otel/opentelemetry-collector:latest` image, and uses the `tonic`'s
`grpc` example as the transport.

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
docker run --rm -it -p 4317:4317 -v $(pwd):/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

On Windows use:

```shell
# From the current directory, run `opentelemetry-collector`
docker run --rm -it -p 4317:4317 -v "%cd%":/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml
```

Run the app which exports logs, metrics and traces via OTLP to the collector

```shell
cargo run
```

## View results

You should be able to see something similar below with different time and ID in the same console that docker runs.

### Span

```text
2024-05-22T20:25:42.892Z    info    TracesExporter  {"kind": "exporter", "data_type": "traces", "name": "logging", "resource spans": 2, "spans": 2}
2024-05-22T20:25:42.892Z    info    ResourceSpans #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeSpans #0
ScopeSpans SchemaURL:
InstrumentationScope basic
InstrumentationScope attributes:
     -> scope-key: Str(scope-value)
Span #0
    Trace ID       : f3f6a43579f63734fe866d34d9aa0b88
    Parent ID      : b66eacd1fcc728d3
    ID             : af01696ea60b9229
    Name           : Sub operation...
    Kind           : Internal
    Start time     : 2024-05-22 20:25:42.877134 +0000 UTC
    End time       : 2024-05-22 20:25:42.8771425 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> another.key: Str(yes)
Events:
SpanEvent #0
     -> Name: Sub span event
     -> Timestamp: 2024-05-22 20:25:42.8771371 +0000 UTC
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
    Trace ID       : f3f6a43579f63734fe866d34d9aa0b88
    Parent ID      :
    ID             : b66eacd1fcc728d3
    Name           : Main operation
    Kind           : Internal
    Start time     : 2024-05-22 20:25:42.8770371 +0000 UTC
    End time       : 2024-05-22 20:25:42.8771505 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> another.key: Str(yes)
Events:
SpanEvent #0
     -> Name: Nice operation!
     -> Timestamp: 2024-05-22 20:25:42.8770471 +0000 UTC
     -> DroppedAttributesCount: 0
     -> Attributes::
          -> bogons: Int(100)
    {"kind": "exporter", "data_type": "traces", "name": "logging"}
```

### Metric

```text
2024-05-22T20:25:42.908Z    info    MetricsExporter {"kind": "exporter", "data_type": "metrics", "name": "logging", "resource metrics": 1, "metrics": 1, "data points": 1}
2024-05-22T20:25:42.908Z    info    ResourceMetrics #0
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
StartTimestamp: 2024-05-22 20:25:42.8767804 +0000 UTC
Timestamp: 2024-05-22 20:25:42.8937799 +0000 UTC
Value: 10
    {"kind": "exporter", "data_type": "metrics", "name": "logging"}
```

### Logs

```text
2024-05-22T20:25:42.914Z    info    LogsExporter    {"kind": "exporter", "data_type": "logs", "name": "logging", "resource logs": 2, "log records": 2}
2024-05-22T20:25:42.914Z    info    ResourceLog #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-appender-tracing 0.4.0
LogRecord #0
ObservedTimestamp: 2024-05-22 20:25:42.8771025 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from banana. My price is 2.99. I am also inside a Span!)
Attributes:
     -> name: Str(my-event-inside-span)
Trace ID: f3f6a43579f63734fe866d34d9aa0b88
Span ID: b66eacd1fcc728d3
Flags: 1
ResourceLog #1
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-appender-tracing 0.4.0
LogRecord #0
ObservedTimestamp: 2024-05-22 20:25:42.8771591 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from apple. My price is 1.99)
Attributes:
     -> name: Str(my-event)
Trace ID:
Span ID:
Flags: 0
    {"kind": "exporter", "data_type": "logs", "name": "logging"}
```
