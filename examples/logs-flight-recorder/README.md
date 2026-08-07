# OpenTelemetry Logs Flight Recorder

The flight recorder keeps recent logs in memory and exports them only when an
application-defined trigger fires. It is useful when detailed logs are valuable
during failures but are unlikely to be useful for successful operations.

Applications can continue producing detailed diagnostic logs without paying the
storage and ingestion cost for every record. Routine logs remain in a bounded
ring buffer and are overwritten as newer records arrive. When an error, timeout,
or other suspicious condition occurs, the application triggers the recorder and
exports the recent context leading up to that condition.

This is an experimental prototype enabled by the
`experimental_logs_flight_recorder` feature.

## How it works

The `FlightRecorderLogProcessor` wraps another `LogProcessor`, such as a
`BatchLogProcessor`:

```text
application logs
      |
      v
FlightRecorderLogProcessor
  bounded in-memory ring buffer
      |
      | only after a trigger
      v
BatchLogProcessor
      |
      v
OTLP exporter
```

Creating the processor also returns a cloneable `FlightRecorderTrigger`. The
handle can be passed to application logic, request handlers, health monitors, or
controllers that know when the retained logs should be exported.

```rust
use opentelemetry_sdk::logs::{
    BatchConfigBuilder, BatchLogProcessor, FlightRecorderLogProcessor,
    SdkLoggerProvider,
};

const MAX_RECORDS: usize = 1_024;

let batch_processor = BatchLogProcessor::builder(exporter)
    .with_batch_config(
        BatchConfigBuilder::default()
            .with_max_queue_size(MAX_RECORDS)
            .build(),
    )
    .build();

let (flight_recorder, trigger) =
    FlightRecorderLogProcessor::builder(batch_processor)
        .with_max_records(MAX_RECORDS)
        .build();

let logger_provider = SdkLoggerProvider::builder()
    .with_log_processor(flight_recorder)
    .build();

// Application code decides when the retained context is valuable.
if operation_failed {
    trigger.trigger()?;
}
```

When wrapping a batch processor, its queue should have at least as many free
slots as the maximum flight-recorder snapshot. This example configures both
limits to the same record count.

## Running the demo

The demo is a small Hyper application that exports triggered logs to an OTLP
HTTP endpoint. Start an OpenTelemetry Collector or another OTLP-compatible
backend, then run:

```shell
OTEL_EXPORTER_OTLP_LOGS_ENDPOINT=http://localhost:4318/v1/logs \
cargo run -p logs-flight-recorder
```

The server listens on `127.0.0.1:3000` by default.

A successful request generates logs but does not export them:

```shell
curl "http://127.0.0.1:3000/work?result=ok&logs=5&request_id=successful"
```

A failing request records its own logs and triggers the current snapshot:

```shell
curl "http://127.0.0.1:3000/work?result=error&logs=5&request_id=failing"
```

The exported snapshot can include logs from the earlier successful request.
This is expected because the prototype uses one application-wide buffer rather
than a separate buffer for each request. Every demo log includes a
`request_id` attribute so interleaved requests can be distinguished.

Configuration:

| Environment variable | Default | Description |
| --- | --- | --- |
| `FLIGHT_RECORDER_LISTEN_ADDR` | `127.0.0.1:3000` | Demo HTTP listen address |
| `FLIGHT_RECORDER_MAX_RECORDS` | `64` | Maximum retained log records |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` | OTLP exporter default | OTLP logs endpoint |

## Trigger ideas

The trigger is deliberately independent of any Rust web framework. Applications
can invoke it when:

- a request or background job fails;
- an operation exceeds a latency threshold;
- a circuit breaker opens;
- a health check detects degradation;
- business logic encounters an unexpected state.

High-severity logs that must always be delivered can use a separate processor.
The flight recorder can then retain verbose contextual logs for selective
export.

## Prototype semantics and limitations

- Capacity is based on record count, not estimated encoded bytes.
- The oldest records are overwritten when the buffer is full.
- Triggering drains the current snapshot. Logs arriving during replay are kept
  for the next trigger.
- `force_flush` also triggers and exports the current snapshot.
- Untriggered records are discarded during shutdown.
- Snapshot handoff is at-most-once and best-effort because the wrapped
  `LogProcessor::emit` API cannot report whether each record was accepted.
- The buffer is application-wide per processor instance. Per-request or
  operation-scoped buffers are not implemented.

