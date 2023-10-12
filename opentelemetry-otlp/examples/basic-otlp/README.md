# Basic OTLP exporter Example

This example shows how to setup OpenTelemetry OTLP exporter for logs, metrics
and traces to exports them to the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP. 
The Collector then sends the data to the appropriate backend, in this case 
Debug Exporter. The Debug Exporter exports data to console.

## Usage

### `docker-compose`

By default runs against the `otel/opentelemetry-collector-dev:latest` image, and uses the `tonic`'s
`grpc` example as the transport.

```shell
docker-compose up
```

In another terminal run the application `cargo run`

The docker-compose terminal will display traces, metrics.

Tear it down:

```shell
docker-compose down
```

### Manual

If you don't want to use `docker-compose`, you can manually run the `otel/opentelemetry-collector` container
and inspect the logs to see traces being transferred.

```shell
# From the current directory, run `opentelemetry-collector`
$ docker run --rm -it -p 4317:4317 -p 4318:4318 -v $(pwd):/cfg otel/opentelemetry-collector:latest --config=/cfg/otel-collector-config.yaml

# Run the app which exports logs, metrics and traces via OTLP to the collector.
$ cargo run
```

## View results

You should be able to see something similar below with different time and ID in the same console that docker runs.

### Span

```text
2023-09-08T21:50:35.884Z        info    ResourceSpans #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-tracing-example)
ScopeSpans #0
ScopeSpans SchemaURL:
InstrumentationScope ex.com/basic
Span #0
    Trace ID       : f8e7ea4dcab43689cea14f708309d682
    Parent ID      : 8b560e2e7238eab5
    ID             : 9e36b48dc07b32fe
    Name           : Sub operation...
    Kind           : Internal
    Start time     : 2023-09-08 21:50:35.872800345 +0000 UTC
    End time       : 2023-09-08 21:50:35.87282574 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> lemons: Str(five)
Events:
SpanEvent #0
     -> Name: Sub span event
     -> Timestamp: 2023-09-08 21:50:35.872808684 +0000 UTC
     -> DroppedAttributesCount: 0
ResourceSpans #1
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-tracing-example)
ScopeSpans #0
ScopeSpans SchemaURL:
InstrumentationScope ex.com/basic
Span #0
    Trace ID       : f8e7ea4dcab43689cea14f708309d682
    Parent ID      :
    ID             : 8b560e2e7238eab5
    Name           : operation
    Kind           : Internal
    Start time     : 2023-09-08 21:50:35.872735497 +0000 UTC
    End time       : 2023-09-08 21:50:35.872832026 +0000 UTC
    Status code    : Unset
    Status message :
Attributes:
     -> ex.com/another: Str(yes)
Events:
SpanEvent #0
     -> Name: Nice operation!
     -> Timestamp: 2023-09-08 21:50:35.872750123 +0000 UTC
     -> DroppedAttributesCount: 0
     -> Attributes::
          -> bogons: Int(100)
        {"kind": "exporter", "data_type": "traces", "name": "logging"}
```

### Metric

```text
2023-09-08T19:14:12.522Z        info    ResourceMetrics #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-metrics-example)
ScopeMetrics #0
ScopeMetrics SchemaURL:
InstrumentationScope ex.com/basic
Metric #0
Descriptor:
     -> Name: ex.com.one
     -> Description: A gauge set to 1.0
     -> Unit:
     -> DataType: Gauge
NumberDataPoints #0
Data point attributes:
     -> A: Str(1)
     -> B: Str(2)
     -> C: Str(3)
     -> lemons: Int(10)
StartTimestamp: 1970-01-01 00:00:00 +0000 UTC
Timestamp: 2023-09-08 19:14:12.468030127 +0000 UTC
Value: 1.000000
Metric #1
Descriptor:
     -> Name: ex.com.two
     -> Description:
     -> Unit:
     -> DataType: Histogram
     -> AggregationTemporality: Cumulative
HistogramDataPoints #0
Data point attributes:
     -> A: Str(1)
     -> B: Str(2)
     -> C: Str(3)
     -> lemons: Int(10)
StartTimestamp: 2023-09-08 19:14:12.466896812 +0000 UTC
Timestamp: 2023-09-08 19:14:12.468052807 +0000 UTC
Count: 1
Sum: 5.500000
Min: 5.500000
Max: 5.500000
ExplicitBounds #0: 0.000000
ExplicitBounds #1: 5.000000
ExplicitBounds #2: 10.000000
ExplicitBounds #3: 25.000000
ExplicitBounds #4: 50.000000
ExplicitBounds #5: 75.000000
ExplicitBounds #6: 100.000000
ExplicitBounds #7: 250.000000
ExplicitBounds #8: 500.000000
ExplicitBounds #9: 750.000000
ExplicitBounds #10: 1000.000000
ExplicitBounds #11: 2500.000000
ExplicitBounds #12: 5000.000000
ExplicitBounds #13: 7500.000000
ExplicitBounds #14: 10000.000000
Buckets #0, Count: 0
Buckets #1, Count: 0
Buckets #2, Count: 1
Buckets #3, Count: 0
Buckets #4, Count: 0
Buckets #5, Count: 0
Buckets #6, Count: 0
Buckets #7, Count: 0
Buckets #8, Count: 0
Buckets #9, Count: 0
Buckets #10, Count: 0
Buckets #11, Count: 0
Buckets #12, Count: 0
Buckets #13, Count: 0
Buckets #14, Count: 0
Buckets #15, Count: 0
HistogramDataPoints #1
StartTimestamp: 2023-09-08 19:14:12.466896812 +0000 UTC
Timestamp: 2023-09-08 19:14:12.468052807 +0000 UTC
Count: 1
Sum: 1.300000
Min: 1.300000
Max: 1.300000
ExplicitBounds #0: 0.000000
ExplicitBounds #1: 5.000000
ExplicitBounds #2: 10.000000
ExplicitBounds #3: 25.000000
ExplicitBounds #4: 50.000000
ExplicitBounds #5: 75.000000
ExplicitBounds #6: 100.000000
ExplicitBounds #7: 250.000000
ExplicitBounds #8: 500.000000
ExplicitBounds #9: 750.000000
ExplicitBounds #10: 1000.000000
ExplicitBounds #11: 2500.000000
ExplicitBounds #12: 5000.000000
ExplicitBounds #13: 7500.000000
ExplicitBounds #14: 10000.000000
Buckets #0, Count: 0
Buckets #1, Count: 1
Buckets #2, Count: 0
Buckets #3, Count: 0
Buckets #4, Count: 0
Buckets #5, Count: 0
Buckets #6, Count: 0
Buckets #7, Count: 0
Buckets #8, Count: 0
Buckets #9, Count: 0
Buckets #10, Count: 0
Buckets #11, Count: 0
Buckets #12, Count: 0
Buckets #13, Count: 0
Buckets #14, Count: 0
Buckets #15, Count: 0
        {"kind": "exporter", "data_type": "metrics", "name": "logging"}
```

### Logs

```text
2023-09-08T21:50:35.884Z        info    ResourceLog #0
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-logging-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-log-appender 0.1.0
LogRecord #0
ObservedTimestamp: 2023-09-08 21:50:35.872759168 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from banana. My price is 2.99. I am also inside a Span!)
Trace ID: f8e7ea4dcab43689cea14f708309d682
Span ID: 8b560e2e7238eab5
Flags: 1
ResourceLog #1
Resource SchemaURL:
Resource attributes:
     -> service.name: Str(basic-otlp-logging-example)
ScopeLogs #0
ScopeLogs SchemaURL:
InstrumentationScope opentelemetry-log-appender 0.1.0
LogRecord #0
ObservedTimestamp: 2023-09-08 21:50:35.872833713 +0000 UTC
Timestamp: 1970-01-01 00:00:00 +0000 UTC
SeverityText: INFO
SeverityNumber: Info(9)
Body: Str(hello from apple. My price is 1.99)
Trace ID:
Span ID:
Flags: 0
```
