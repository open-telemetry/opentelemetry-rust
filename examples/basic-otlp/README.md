# Basic OTLP exporter Example

This example shows basic span and metric usage, and exports to the [OpenTelemetry Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP.

## Usage

```shell
# Run `opentelemetry-collector`
$ docker run  -p4317:4317 otel/opentelemetry-collector:latest

# Report spans/metrics
$ cargo run
```

# View result

You should be able to see something similar below in the same console when you run docker

## Span

```
Resource labels:
     -> service.name: STRING(unknown_service)
InstrumentationLibrarySpans #0
InstrumentationLibrary
Span #0
    Trace ID       : e60184475358d966b0880ec1cc1cf515
    Parent ID      : 7b74443b55e404ba
    ID             : f8e4a037c0fa63bd
    Name           : Sub operation...
    Kind           : SPAN_KIND_INTERNAL
    Start time     : 2021-11-19 04:07:46.29597 +0000 UTC
    End time       : 2021-11-19 04:07:46.295997 +0000 UTC
    Status code    : STATUS_CODE_UNSET
    Status message :
Attributes:
     -> lemons: STRING(five)
Events:
SpanEvent #0
     -> Name: Sub span event
     -> Timestamp: 2021-11-19 04:07:46.295982 +0000 UTC
     -> DroppedAttributesCount: 0
ResourceSpans #1
Resource labels:
     -> service.name: STRING(unknown_service)
InstrumentationLibrarySpans #0
InstrumentationLibrary
Span #0
    Trace ID       : e60184475358d966b0880ec1cc1cf515
    Parent ID      :
    ID             : 7b74443b55e404ba
    Name           : operation
    Kind           : SPAN_KIND_INTERNAL
    Start time     : 2021-11-19 04:07:46.295898 +0000 UTC
    End time       : 2021-11-19 04:07:46.296015 +0000 UTC
    Status code    : STATUS_CODE_UNSET
    Status message :
Attributes:
     -> ex.com/another: STRING(yes)
Events:
SpanEvent #0
     -> Name: Nice operation!
     -> Timestamp: 2021-11-19 04:07:46.295915 +0000 UTC
     -> DroppedAttributesCount: 0
     -> Attributes:
         -> bogons: INT(100)
```

## Metric

```
2021-11-19T04:08:36.453Z	INFO	loggingexporter/logging_exporter.go:56	MetricsExporter	{"#metrics": 1}
2021-11-19T04:08:36.454Z	DEBUG	loggingexporter/logging_exporter.go:66	ResourceMetrics #0
Resource labels:
     -> service.name: STRING(unknown_service)
InstrumentationLibraryMetrics #0
InstrumentationLibrary ex.com/basic
Metric #0
Descriptor:
     -> Name: ex.com.one
     -> Description: A ValueObserver set to 1.0
     -> Unit:
     -> DataType: Gauge
NumberDataPoints #0
Data point attributes:
     -> A: STRING(1)
     -> B: STRING(2)
     -> C: STRING(3)
     -> lemons: INT(10)
StartTimestamp: 2021-11-19 04:07:46.29555 +0000 UTC
Timestamp: 2021-11-19 04:08:36.297279 +0000 UTC
Value: 1.000000
```