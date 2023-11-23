![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Rust

The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.

[![Crates.io: opentelemetry](https://img.shields.io/crates/v/opentelemetry.svg)](https://crates.io/crates/opentelemetry)
[![Documentation](https://docs.rs/opentelemetry/badge.svg)](https://docs.rs/opentelemetry)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![codecov](https://codecov.io/gh/open-telemetry/opentelemetry-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/open-telemetry/opentelemetry-rust)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

[Website](https://opentelemetry.io/) |
[Slack](https://cloud-native.slack.com/archives/C03GDP0H023) |
[Documentation](https://docs.rs/opentelemetry)

## Overview

OpenTelemetry is a collection of tools, APIs, and SDKs used to instrument,
generate, collect, and export telemetry data (metrics, logs, and traces) for
analysis in order to understand your software's performance and behavior. You
can export and analyze them using [Prometheus], [Jaeger], and other
observability tools.

*Compiler support: [requires `rustc` 1.65+][msrv]*

[Prometheus]: https://prometheus.io
[Jaeger]: https://www.jaegertracing.io
[msrv]: #supported-rust-versions

## Project Status

| Signal  | Status     |
| ------- | ---------- |
| Logs    | Alpha*     |
| Metrics | Alpha      |
| Traces  | Beta       |

*OpenTelemetry Rust is not introducing a new end user callable Logging API.
Instead, it provides [Logs Bridge
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/bridge-api.md),
that allows one to write log appenders that can bridge existing logging
libraries to the OpenTelemetry log data model. The following log appenders are
available:

* [opentelemetry-appender-log](opentelemetry-appender-log/README.md)
* [opentelemetry-appender-tracing](opentelemetry-appender-tracing/README.md)
* opentelemetry-appender-slog // TODO: Add link once available

If you already use the logging APIs from above, continue to use them, and use
the appenders above to bridge the logs to OpenTelemetry. If you are using a
library not listed here, feel free to contribute a new appender for the same.

If you are starting fresh, then consider using
[tracing](https://github.com/tokio-rs/tracing) as your logging API. It supports
structured logging and is actively maintained.

Project versioning information and stability guarantees can be found
[here](VERSIONING.md).

## Getting Started

```rust
use opentelemetry::{
    global,
    sdk::trace::TracerProvider,
    trace::{Tracer, TracerProvider as _},
};

fn main() {
    // Create a new trace pipeline that prints to stdout
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("readme_example");

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    // Shutdown trace pipeline
    global::shutdown_tracer_provider();
}
```

See the [examples](./examples) directory for different integration patterns.

## Ecosystem

### Related Crates

In addition to `opentelemetry`, the [`open-telemetry/opentelemetry-rust`]
repository contains several additional crates designed to be used with the
`opentelemetry` ecosystem. This includes a collection of trace `SpanExporter`
and metrics pull and push controller implementations, as well as utility and
adapter crates to assist in propagating state and instrumenting applications.

In particular, the following crates are likely to be of interest:

* [`opentelemetry-aws`] provides unofficial propagators for AWS X-ray.
* [`opentelemetry-datadog`] provides additional exporters to [`Datadog`].
* [`opentelemetry-dynatrace`] provides additional exporters to Dynatrace.
* [`opentelemetry-contrib`] provides additional exporters and propagators that
  are experimental.
* [`opentelemetry-http`] provides an interface for injecting and extracting
  trace information from [`http`] headers.
* [`opentelemetry-jaeger`] provides context propagation using [jaeger propagation format](https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format).
* [`opentelemetry-otlp`] exporter for sending trace and metric data in the OTLP
  format to the OpenTelemetry collector.
* [`opentelemetry-prometheus`] provides a pipeline and exporter for sending
  metrics information to [`Prometheus`].
* [`opentelemetry-semantic-conventions`] provides standard names and semantic
  otel conventions.
* [`opentelemetry-stackdriver`] provides an exporter for Google's [Cloud Trace]
  (which used to be called StackDriver).
* [`opentelemetry-zipkin`] provides a pipeline and exporter for sending trace
  information to [`Zipkin`].

Additionally, there are also several third-party crates which are not
maintained by the `opentelemetry` project. These include:

* [`tracing-opentelemetry`] provides integration for applications instrumented
  using the [`tracing`] API and ecosystem.
* [`actix-web-opentelemetry`] provides integration for the [`actix-web`] web
  server and ecosystem.
* [`opentelemetry-application-insights`] provides an unofficial [Azure
  Application Insights] exporter.
* [`opentelemetry-tide`] provides integration for the [`Tide`] web server and
  ecosystem.

If you're the maintainer of an `opentelemetry` ecosystem crate not listed
above, please let us know! We'd love to add your project to the list!

[`open-telemetry/opentelemetry-rust`]: https://github.com/open-telemetry/opentelemetry-rust
[`opentelemetry-jaeger`]: https://crates.io/crates/opentelemetry-jaeger
[`Jaeger`]: https://www.jaegertracing.io
[`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
[`opentelemetry-http`]: https://crates.io/crates/opentelemetry-http
[`opentelemetry-prometheus`]: https://crates.io/crates/opentelemetry-prometheus
[`opentelemetry-aws`]: https://crates.io/crates/opentelemetry-aws
[`Prometheus`]: https://prometheus.io
[`opentelemetry-zipkin`]: https://crates.io/crates/opentelemetry-zipkin
[`Zipkin`]: https://zipkin.io
[`opentelemetry-contrib`]: https://crates.io/crates/opentelemetry-contrib
[`Datadog`]: https://www.datadoghq.com
[`opentelemetry-datadog`]: https://crates.io/crates/opentelemetry-datadog
[`opentelemetry-dynatrace`]: https://crates.io/crates/opentelemetry-dynatrace
[`opentelemetry-semantic-conventions`]: https://crates.io/crates/opentelemetry-semantic-conventions
[`http`]: https://crates.io/crates/http

[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[`tracing`]: https://crates.io/crates/tracing
[`actix-web-opentelemetry`]: https://crates.io/crates/actix-web-opentelemetry
[`actix-web`]: https://crates.io/crates/actix-web
[`opentelemetry-application-insights`]: https://crates.io/crates/opentelemetry-application-insights
[Azure Application Insights]: https://docs.microsoft.com/en-us/azure/azure-monitor/app/app-insights-overview
[`opentelemetry-tide`]: https://crates.io/crates/opentelemetry-tide
[`Tide`]: https://crates.io/crates/tide
[`opentelemetry-stackdriver`]: https://crates.io/crates/opentelemetry-stackdriver
[Cloud Trace]: https://cloud.google.com/trace/

## Supported Rust Versions

OpenTelemetry is built against the latest stable release. The minimum supported
version is 1.64. The current OpenTelemetry version is not guaranteed to build
on Rust versions earlier than the minimum supported version.

The current stable Rust compiler and the three most recent minor versions
before it will always be supported. For example, if the current stable compiler
version is 1.49, the minimum supported version will not be increased past 1.46,
three minor versions prior. Increasing the minimum supported compiler version
is not considered a semver breaking change as long as doing so complies with
this policy.

## Contributing

See the [contributing file](CONTRIBUTING.md).

The Rust special interest group (SIG) meets weekly on Tuesdays at 8 AM Pacific
Time (16:00 UTC). The meeting is subject to change depending on contributors'
availability. Check the [OpenTelemetry community
calendar](https://calendar.google.com/calendar/embed?src=google.com_b79e3e90j7bbsa2n2p5an5lf60%40group.calendar.google.com)
for specific dates and for Zoom meeting links. "OTel Rust SIG" is the name of
meeting for this group.

Meeting notes are available as a public [Google
doc](https://docs.google.com/document/d/1tGKuCsSnyT2McDncVJrMgg74_z8V06riWZa0Sr79I_4/edit).
If you have trouble accessing the doc, please get in touch on
[Slack](https://cloud-native.slack.com/archives/C03GDP0H023).

The meeting is open for all to join. We invite everyone to join our meeting,
regardless of your experience level. Whether you're a seasoned OpenTelemetry
developer, just starting your journey, or simply curious about the work we do,
you're more than welcome to participate!
