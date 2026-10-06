//! # OpenTelemetry Protocol File Exporter
//!
//! Exporters that write traces, metrics and logs as [OTLP JSON] lines to stdout, to a file, or
//! to any other [`std::io::Write`] implementation, as described by the [OTLP File Exporter]
//! specification.
//!
//! <div class="warning">The OTLP File Exporter specification has
//! <a href="https://opentelemetry.io/docs/specs/otel/document-status/">Development</a> status,
//! so the configuration API of this crate may change. The output is the stable OTLP JSON
//! encoding.</div>
//!
//! Use these exporters where sending OTLP over the network is not possible or not wanted:
//!
//! * Function-as-a-Service platforms, such as AWS Lambda, which capture stdout as the
//!   function's log stream.
//! * Containers whose stdout is already collected by a log scraping pipeline.
//! * Hosts that persist telemetry to local files for reliability.
//!
//! ## Output format
//!
//! Every export is written as one [JSON Lines] entry: a single line of compact, UTF-8 encoded
//! JSON terminated by `\n`.
//!
//! * [`SpanExporter`] writes one `TracesData` object per batch of spans.
//! * [`MetricExporter`] writes one `MetricsData` object per collection.
//! * [`LogExporter`] writes one `LogsData` object per batch of log records.
//!
//! Any OTLP JSON consumer can read these lines, for example the OpenTelemetry Collector's
//! [OTLP JSON file receiver].
//!
//! The specification requires a file to contain a single type of telemetry, so give each
//! signal its own file. On stdout, the lines share the stream with anything else the process
//! prints; consumers can tell them apart by their top-level key: `resourceSpans`,
//! `resourceMetrics` or `resourceLogs`.
//!
//! ## Getting started
//!
//! ```no_run
//! # #[cfg(all(feature = "trace", feature = "metrics", feature = "logs"))]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use opentelemetry::global;
//! use opentelemetry_otlp_file::{LogExporter, MetricExporter, SpanExporter};
//! use opentelemetry_sdk::logs::SdkLoggerProvider;
//! use opentelemetry_sdk::metrics::SdkMeterProvider;
//! use opentelemetry_sdk::trace::SdkTracerProvider;
//!
//! // Spans are written to stdout, the default output.
//! let tracer_provider = SdkTracerProvider::builder()
//!     .with_batch_exporter(SpanExporter::default())
//!     .build();
//! global::set_tracer_provider(tracer_provider.clone());
//!
//! // Metrics are appended to a file, which is created if it does not exist.
//! let metric_exporter = MetricExporter::builder()
//!     .with_file("/var/log/otel/metrics.jsonl")
//!     .build()?;
//! let meter_provider = SdkMeterProvider::builder()
//!     .with_periodic_exporter(metric_exporter)
//!     .build();
//! global::set_meter_provider(meter_provider.clone());
//!
//! // Log records can be written to any `std::io::Write` implementation. Connect the logger
//! // provider to your logging library with an appender such as
//! // `opentelemetry-appender-tracing`.
//! let log_exporter = LogExporter::builder()
//!     .with_writer(std::io::stderr())
//!     .build()?;
//! let logger_provider = SdkLoggerProvider::builder()
//!     .with_batch_exporter(log_exporter)
//!     .build();
//!
//! // ... run the instrumented application ...
//!
//! tracer_provider.shutdown()?;
//! meter_provider.shutdown()?;
//! logger_provider.shutdown()?;
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "trace", feature = "metrics", feature = "logs")))]
//! # fn main() {}
//! ```
//!
//! ## Writing behavior
//!
//! * Files are opened in append mode when the exporter is built, so a missing parent directory
//!   or a permission problem is reported by `build`.
//! * Each line is written with a single `write_all` call and then flushed, so a line is
//!   complete as soon as the export call returns and is not interleaved with other output
//!   written through Rust's `stdout`, such as `println!`.
//! * Shutting down an exporter flushes and drops its writer, which closes a file opened by the
//!   exporter. Exports after shutdown fail with
//!   [`OTelSdkError::AlreadyShutdown`](opentelemetry_sdk::error::OTelSdkError::AlreadyShutdown).
//!
//! ## Processors and short-lived environments
//!
//! The specification recommends pairing these exporters with a batching processor, as in the
//! example above. Batch processors and the periodic reader export from background threads,
//! which do not run while an environment such as AWS Lambda has suspended the process between
//! requests. Call `force_flush` on the providers at the end of each request, or flush them when
//! the environment shuts down; the [README] compares these approaches on AWS Lambda.
//!
//! [README]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry-otlp-file#aws-lambda-and-other-short-lived-environments
//!
//! ## Configuration
//!
//! The Rust SDK does not select exporters from the `OTEL_TRACES_EXPORTER`,
//! `OTEL_METRICS_EXPORTER` and `OTEL_LOGS_EXPORTER` environment variables, so these exporters
//! are configured in code.
//!
//! [`MetricExporter`] reads `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`, which accepts
//! `cumulative` (the default), `delta` or `lowmemory`.
//! [`MetricExporterBuilder::with_temporality`] takes precedence over it.
//! `OTEL_EXPORTER_OTLP_METRICS_DEFAULT_HISTOGRAM_AGGREGATION` is not supported, because the SDK
//! does not let exporters choose the default aggregation; use a view to select the histogram
//! aggregation instead.
//!
//! ## Feature flags
//!
//! * `trace`: the span exporter. Enabled by default.
//! * `metrics`: the metric exporter. Enabled by default.
//! * `logs`: the log exporter. Enabled by default.
//! * `internal-logs`: report problems, such as an invalid environment variable, through the
//!   OpenTelemetry internal logging macros. Enabled by default.
//!
//! [OTLP JSON]: https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding
//! [OTLP File Exporter]: https://opentelemetry.io/docs/specs/otel/protocol/file-exporter/
//! [JSON Lines]: https://jsonlines.org
//! [OTLP JSON file receiver]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/otlpjsonfilereceiver
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(test, deny(warnings))]

mod error;
#[cfg(feature = "logs")]
mod logs;
#[cfg(feature = "metrics")]
mod metrics;
#[cfg(any(feature = "trace", feature = "metrics", feature = "logs"))]
mod output;
#[cfg(feature = "trace")]
mod trace;

pub use error::ExporterBuildError;

#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub use logs::{LogExporter, LogExporterBuilder};

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub use metrics::{MetricExporter, MetricExporterBuilder};

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub use trace::{SpanExporter, SpanExporterBuilder};
