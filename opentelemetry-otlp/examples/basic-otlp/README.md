# Basic OTLP Exporter Example

This example demonstrates how to set up an OpenTelemetry OTLP exporter for logs,
metrics, and traces to send data to the [OpenTelemetry
Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP
over gRPC. The Collector then forwards the data to the configured backend, which
in this case is the logging exporter, displaying data on the console.
Additionally, the example configures a `tracing::fmt` layer to output logs
emitted via `tracing` to `stdout`. For demonstration, this layer uses a filter
to display `DEBUG` level logs from various OpenTelemetry components. In real
applications, these filters should be adjusted appropriately.

The example employs a `BatchExporter` for logs and traces, which is the
recommended approach when using OTLP exporters. While it can be modified to use
a `SimpleExporter`, this requires the main method to be a `tokio::main` function
since the `tonic` client requires a Tokio runtime. If you prefer not to use
`tokio::main`, then the `init_logs` and `init_traces` functions must be executed
within a Tokio runtime.

This examples uses the default `PeriodicReader` for metrics, which uses own
thread for background processing/exporting. Since the `tonic` client requires a
Tokio runtime, the main method must be a `tokio::main` function. If you prefer not
to use `tokio::main`, then the `init_metrics` function must be executed within a
Tokio runtime.

Below is an example on how to use non `tokio::main`:

```rust
fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
     let rt = tokio::runtime::Runtime::new()?;
     let tracer_provider = rt.block_on(async {
          init_traces()
     })?;
     global::set_tracer_provider(tracer_provider.clone());

     let meter_provider = rt.block_on(async {
          init_metrics()
     })?;
     global::set_meter_provider(meter_provider.clone());

     let logger_provider = rt.block_on(async {
          init_logs()
     })?;

     // Ensure the runtime (`rt`) remains active until the program ends
     // Additional code goes here...
}
```

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
          -> some.key: Int(100)
    {"kind": "exporter", "data_type": "traces", "name": "logging"}
```

### Metric

```text
2024-05-22T20:25:42.908Z    info    MetricExporter {"kind": "exporter", "data_type": "metrics", "name": "logging", "resource metrics": 1, "metrics": 1, "data points": 1}
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
2024-05-22T20:25:42.914Z    info    LogExporter    {"kind": "exporter", "data_type": "logs", "name": "logging", "resource logs": 2, "log records": 2}
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
