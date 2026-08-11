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

The SDK prototype provides two variants:

- `FlightRecorderLogProcessor` maintains one application-wide buffer.
- `ScopedFlightRecorderLogProcessor` maintains an isolated buffer for each
  explicitly created operation scope.

The demo uses scoped recording to model concurrent HTTP requests without
coupling the SDK API to a particular Rust web framework.

## How scoped recording works

The `ScopedFlightRecorderLogProcessor` wraps another `LogProcessor`, such as a
`BatchLogProcessor`:

```text
application logs
      |
      v
ScopedFlightRecorderLogProcessor
  INFO and lower with context: per-operation ring buffer
  WARN and higher: direct path
  logs without a scope: direct path
      |
      | only after a trigger
      v
BatchLogProcessor
      |
      v
OTLP exporter
```

Creating the processor also returns a cloneable `ScopedFlightRecorder` handle.
Applications use it to create a scope at a request, job, RPC, or message
boundary.

```rust
use opentelemetry_sdk::logs::{
    BatchConfigBuilder, BatchLogProcessor, ScopedFlightRecorderLogProcessor,
    ScopedFlightRecorderOverflowPolicy, SdkLoggerProvider,
};
use opentelemetry::logs::Severity;

const MAX_RECORDS: usize = 256;

let batch_processor = BatchLogProcessor::builder(exporter)
    .with_batch_config(
        BatchConfigBuilder::default()
            .with_max_queue_size(MAX_RECORDS)
            .build(),
    )
    .build();

let (flight_recorder, recorder) =
    ScopedFlightRecorderLogProcessor::builder(batch_processor)
        .with_max_records_per_scope(MAX_RECORDS)
        .with_max_active_scopes(128)
        .with_max_buffer_size_bytes_per_scope(256 * 1024)
        .with_max_total_buffer_size_bytes(16 * 1024 * 1024)
        .with_max_record_size_bytes(64 * 1024)
        .with_overflow_policy(ScopedFlightRecorderOverflowPolicy::Drop)
        .with_max_buffered_severity(Severity::Info4)
        .build();

let logger_provider = SdkLoggerProvider::builder()
    .with_log_processor(flight_recorder)
    .build();

let operation = recorder.try_start()?;
let result = operation
    .with_context(async {
        run_operation().await
    })
    .await;

if result.is_err() {
    operation.trigger()?;
} else {
    operation.discard();
}
```

`trigger()` waits for the wrapped processor's `force_flush` and is appropriate
when completion must be observed before continuing. `handoff()` only submits the
snapshot to the wrapped processor. It avoids waiting for exporter completion,
but remains synchronous around recorder locks and delegate `emit` calls. Both
operations are serialized with other triggers. Because `handoff()` does not
pre-flush a bounded delegate queue, the application must account for records
already queued when sizing that queue. It also does not preserve submission
order between records already on the delegate's normal path and the older
snapshot: consumers should use record timestamps for ordering. `trigger()`
pre-flushes normal-path records before replay, but those newer records can
therefore still arrive before the older snapshot.

When wrapping a batch processor, its queue should have at least as many free
slots as the maximum snapshot from one scope. This example configures both
limits to the same record count. Estimated byte limits provide the primary
memory bounds: each scope defaults to 256 KiB, all scopes share a 16 MiB
aggregate budget, and records estimated above 64 KiB bypass buffering.

## Running the demo

The demo is a small Hyper application that exports triggered logs to an OTLP
HTTP endpoint. Start an OpenTelemetry Collector or another OTLP-compatible
backend, then run:

```shell
OTEL_EXPORTER_OTLP_LOGS_ENDPOINT=http://localhost:4318/v1/logs \
cargo run -p logs-flight-recorder
```

The server listens on `127.0.0.1:3000` by default.

A successful request generates scoped INFO logs and discards them when the
request completes:

```shell
curl "http://127.0.0.1:3000/work?result=ok&logs=5&request_id=successful"
```

A warning request demonstrates the normal export path. Its INFO records are
discarded with the successful scope, while its WARN record is handed directly
to the batch processor:

```shell
curl "http://127.0.0.1:3000/work?result=warn&logs=5&request_id=warning"
```

A failing request records its own logs and triggers the current snapshot:

```shell
curl "http://127.0.0.1:3000/work?result=error&logs=5&request_id=failing"
```

Only the failing request's buffered INFO records are exported. Logs from
successful or concurrently executing requests remain isolated in their own
scopes.

Use `delay_ms` to make concurrent request isolation easy to observe:

```shell
curl "http://127.0.0.1:3000/work?result=ok&logs=5&delay_ms=200&request_id=slow-success" &
curl "http://127.0.0.1:3000/work?result=error&logs=2&request_id=failure"
```

The failure snapshot contains only records with `request_id=failure`.

Configuration:

| Environment variable | Default | Description |
| --- | --- | --- |
| `FLIGHT_RECORDER_LISTEN_ADDR` | `127.0.0.1:3000` | Demo HTTP listen address |
| `FLIGHT_RECORDER_MAX_RECORDS` | `64` | Maximum retained records per request |
| `FLIGHT_RECORDER_MAX_ACTIVE_SCOPES` | `128` | Maximum concurrent request scopes |
| `FLIGHT_RECORDER_MAX_BUFFER_BYTES_PER_SCOPE` | `262144` | Maximum estimated retained bytes per request |
| `FLIGHT_RECORDER_MAX_TOTAL_BUFFER_BYTES` | `16777216` | Aggregate estimated retained bytes across requests |
| `FLIGHT_RECORDER_MAX_RECORD_BYTES` | `65536` | Records above this estimated size bypass buffering |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` | OTLP exporter default | OTLP logs endpoint |

## Trigger ideas

Scopes are deliberately independent of any Rust web framework. Applications
can create and trigger them around:

- a request or background job fails;
- an operation exceeds a latency threshold;
- a circuit breaker opens;
- a health check detects degradation;
- business logic encounters an unexpected state.

By default, the flight recorder buffers the TRACE, DEBUG, and INFO severity
ranges. WARN, ERROR, and FATAL records bypass the ring buffer and follow the
wrapped processor's normal export path. The maximum buffered severity is
configurable.

## Prototype semantics and limitations

- Capacity is bounded by both record count and a conservative in-memory size
  estimate. This is not the serialized OTLP size.
- When the aggregate scoped byte budget is exhausted, new low-severity records
  are dropped by default, preserving the expected ingestion-cost bound. Set
  `ScopedFlightRecorderOverflowPolicy::Export` to send them through the wrapped
  processor instead.
- Scope admission reports whether the active-scope limit was reached, the
  processor was shut down, or an internal registry failure occurred.
- INFO and lower-severity records are buffered by default; WARN and higher
  records bypass the buffer.
- The oldest records are overwritten when the buffer is full.
- Each operation scope has its own ring buffer.
- Triggering drains that scope and switches it to passthrough so subsequent
  records follow the normal path.
- `handoff` submits a snapshot without flushing; `trigger` additionally
  pre-flushes and flushes after replay.
- Dropping or discarding an untriggered scope loses its buffered records.
- Logs without an attached scope follow the normal path.
- Independently spawned tasks must explicitly use the same scope's
  `with_context` wrapper.
- `force_flush` triggers and exports all currently active scoped snapshots.
- Untriggered records are discarded during shutdown.
- Snapshot handoff is at-most-once and best-effort because the wrapped
  `LogProcessor::emit` API cannot report whether each record was accepted.
- The global `FlightRecorderLogProcessor` remains available when one shared
  application-wide history is preferred.
