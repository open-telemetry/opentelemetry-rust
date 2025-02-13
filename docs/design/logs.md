# OpenTelemetry Rust Logs Design

Status:
[Development](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/document-status.md)

## Overview

OpenTelemetry (OTel) Logs support differs from Metrics and Traces as it does not
introduce a new logging API for end users. Instead, OTel recommends leveraging
existing logging libraries such as `log` and `tracing`, while providing bridges
(appenders) to route logs through OpenTelemetry.

Unlike Traces and Metrics, which introduced new APIs, Logs took a different
approach due to the long history of existing logging solutions. In Rust, the
most widely used logging libraries are `log` and `tracing`. OTel Rust maintains
appenders for these libraries, allowing users to seamlessly integrate with
OpenTelemetry without changing their existing logging instrumentation.

The `tracing` appender is particularly optimized for performance due to its
widespread adoption and the fact that `tracing` itself has a bridge from the
`log` crate. Notably, OpenTelemetry Rust itself is instrumented using `tracing`
for internal logs. Additionally, when OTel began supporting logging as a signal,
the `log` crate lacked structured logging support, reinforcing the decision to
prioritize `tracing`.

## Benefits of OpenTelemetry Logs

- **Unified configuration** across Traces, Metrics, and Logs.
- **Automatic correlation** with Traces.
- **Consistent Resource attributes** across signals.
- **Multiple destinations support**: Logs can continue flowing to existing
  destinations like stdout while also being sent to an OpenTelemetry-capable
  backend, typically via an OTLP Exporter or exporters that export to operating
  system native systems like `Windows ETW` or `Linux user_events`.
- **Standalone logging support** for applications that use OpenTelemetry as
  their primary logging mechanism.

## Key Design Principles

- High performance - no locks/contention in the hot path, minimal/no heap
  allocation.
- Capped resource usage - well-defined behavior when overloaded.
- Self-observable.
- Well defined Error handling, returning Result as appropriate instead of panic.
- Minimal public API, exposing based on need only.

## Logs API

The OTel Logs API is not intended for direct end-user usage. Instead, it is
designed for appender/bridge authors to integrate existing logging libraries
with OpenTelemetry. However, there is nothing preventing it from being used by
end-users.

### API Components

1. **Key-Value Structs**: Used in `LogRecord`, where keys are shared across
   signals but values differ from Metrics and Traces. This is because values in
   Logs can contain more complex structures than those in Traces and Metrics.
2. **Traits**:
    - `LoggerProvider` - provides methods to obtain Logger.
    - `Logger` - provides methods to create LogRecord and emit the created
      LogRecord.
    - `LogRecord` - provides methods to populate LogRecord.
3. **No-Op Implementations**: By default, the API performs no operations until
   an SDK is attached.

### Logs Flow

1. Obtain a `LoggerProvider` implementation.
2. Use the `LoggerProvider` to create `Logger` instances, specifying a scope
   name (module/component emitting logs). Optional attributes and version are
   also supported.
3. Use the `Logger` to create an empty `LogRecord` instance.
4. Populate the `LogRecord` with body, timestamp, attributes, etc.
5. Call `Logger.emit(LogRecord)` to process and export the log.

If only the Logs API is used (without an SDK), all the above steps result in no
operations, following OpenTelemetry’s philosophy of separating API from SDK. The
official Logs SDK provides real implementations to process and export logs.
Users or vendors can also provide alternative SDK implementations.

## Logs SDK

The OpenTelemetry Logs SDK provides an OTel specification-compliant
implementation of the Logs API, handling log processing and export.

### Core Components

#### `SdkLoggerProvider`

- Implements the `LoggerProvider` trait.
- Creates and manages `SdkLogger` instances.
- Holds logging configuration, including `Resource` and processors.
- Does not retain a list of created loggers. Instead, it passes an owned clone
  of itself to each logger created. This is done so that loggers get a hold of
  the configuration (like which processor to invoke).
- Uses an `Arc<LoggerProviderInner>` and delegates all configuration to
  `LoggerProviderInner`. This allows cheap cloning of itself and ensures all
  clones point to the same underlying configuration.
- As `SdkLoggerProvider` only holds an `Arc` of its inner, it can only accept
  `&self` in its methods like flush and shutdown. Else it needs to rely on
  interior mutability that comes with runtime performance costs. Since methods
  like shutdown usually need to mutate interior state, components like exporter
  use interior mutability to handle shutdown. (More on this in the exporter
  section)
- `LoggerProviderInner` implements `Drop`, triggering `shutdown()` when no
  references remain. However, in practice, loggers are often stored statically
  inside appenders (like tracing-appender), so explicit shutdown by the user is
  required.

#### `SdkLogger`

- Implements the `Logger` trait.
- Creates `SdkLogRecord` instances and emits them.
- Calls `OnEmit()` on all registered processors when emitting logs.
- Passes mutable references to each processor (`&mut log_record`), i.e.,
  ownership is not passed to the processor. This ensures that the logger avoids
  cloning costs. Since a mutable reference is passed, processors can modify the
  log, and it will be visible to the next processor in the chain.
- Since the processor only gets a reference to the log, it cannot store it
  beyond the `OnEmit()`. If a processor needs to buffer logs, it must explicitly
  copy them to the heap.
- This design allows for stack-only log processing when exporting to operating
  system native facilities like `Windows ETW` or `Linux user_events`.
- OTLP Exporting requires network calls (HTTP/gRPC) and batching of logs for
  efficiency purposes. These exporters buffer log records by copying them to the
  heap. (More on this in the BatchLogRecordProcessor section)

#### `LogRecord`

- Holds log data, including attributes.
- Uses an inline array for up to 5 attributes to optimize stack usage.
- Falls back to a heap-allocated `Vec` if more attributes are required.
- Inspired by Go’s `slog` library for efficiency.

#### LogRecord Processors

`SdkLoggerProvider` allows being configured with any number of LogProcessors.
They get called in the order of registration. Log records are passed to the
`OnEmit` method of LogProcessor. LogProcessors can be used to process the log
records, enrich them, filter them, and export to destinations by leveraging
LogRecord Exporters.

Following built-in Log processors are provided in the Log SDK:

##### SimpleLogProcessor

This processor is designed to be used for exporting purposes. Export is handled
by an Exporter (which is a separate component). SimpleLogProcessor is "simple"
in the sense that it does not attempt to do any processing - it just calls the
exporter and passes the log record to it. To comply with OTel specification, it
synchronizes calls to the `Export()` method, i.e., only one `Export()` call will
be done at any given time.

SimpleLogProcessor is only used for test/learning purposes and is often used
along with a `stdout` exporter.

##### BatchLogProcessor

This is another "exporting" processor. As with SimpleLogProcessor, a different
component named LogExporter handles the actual export logic. BatchLogProcessor
buffers/batches the logs it receives into an in-memory buffer. It invokes the
exporter every 1 second or when 512 items are in the batch (customizable). It
uses a background thread to do the export, and communication between the user
thread (where logs are emitted) and the background thread occurs with `mpsc`
channels.

The max amount of items the buffer holds is 2048 (customizable). Once the limit
is reached, any *new* logs are dropped. It *does not* apply back-pressure to the
user thread and instead drops logs.

As with SimpleLogProcessor, this component also ensures only one export is
active at a given time. A modified version of this is required to achieve higher
throughput in some environments.

In this design, at most 2048+512 logs can be in memory at any given point. In
other words, that many logs can be lost if the app crashes in the middle.

## LogExporters

LogExporters are responsible for exporting logs to a destination. Some of them
include:

1. **InMemoryExporter** - exports to an in-memory list, primarily for
   unit-testing. This is used extensively in the repo itself, and external users
   are also encouraged to use this.
2. **Stdout exporter** - prints telemetry to stdout. Only for debugging/learning
   purposes. The output format is not defined and also is not performance
   optimized. A production-recommended version with a standardized output format
   is in the plan.
3. **OTLP Exporter** - OTel's official exporter which uses the OTLP protocol
   that is designed with the OTel data model in mind. Both HTTP and gRPC-based
   exporting is offered.
4. **Exporters to OS Kernel facilities** - These exporters are not maintained in
   the core repo but listed for completion. They export telemetry to Windows ETW
   or Linux user_events. They are designed for high-performance workloads. Due
   to their nature of synchronous exporting, they do not require
   buffering/batching. This allows logs to operate entirely on the stack and can
   scale easily with the number of CPU cores. (Kernel uses per-CPU buffers for
   the events, ensuring no contention)

## `tracing` Log Appender

The `tracing` appender bridges `tracing` logs to OpenTelemetry. Logs emitted via
`tracing` macros (`info!`, `warn!`, etc.) are forwarded to OpenTelemetry through
this integration.

- `tracing` is designed for high performance, using *layers* or *subscribers* to
  handle emitted logs (events).
- The appender implements a `Layer`, receiving logs from `tracing`.
- Uses the OTel Logs API to create `LogRecord`, populate it, and emit it via
  `Logger.emit(LogRecord)`.
- If no Logs SDK is present, the process is a no-op.

## Summary

- OpenTelemetry Logs does not provide a user-facing logging API.
- Instead, it integrates with existing logging libraries (`log`, `tracing`).
- The Logs API defines key traits but performs no operations unless an SDK is
  installed.
- The Logs SDK enables log processing, transformation, and export.
- The Logs SDK is performance optimized to minimize copying and heap allocation,
  wherever feasible.
- The `tracing` appender efficiently routes logs to OpenTelemetry without
  modifying existing logging workflows.
