# OpenTelemetry Rust

The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.

[![Crates.io: opentelemetry](https://img.shields.io/crates/v/opentelemetry.svg)](https://crates.io/crates/opentelemetry)
[![Documentation](https://docs.rs/opentelemetry/badge.svg)](https://docs.rs/opentelemetry)
[![LICENSE](https://img.shields.io/crates/l/opentelemetry)](./LICENSE)
[![GitHub Actions CI](https://github.com/open-telemetry/opentelemetry-rust/workflows/CI/badge.svg)](https://github.com/open-telemetry/opentelemetry-rust/actions?query=workflow%3ACI+branch%3Amain)
[![codecov](https://codecov.io/gh/open-telemetry/opentelemetry-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/open-telemetry/opentelemetry-rust)
[![Slack](https://img.shields.io/badge/slack-@cncf/otel/rust-brightgreen.svg?logo=slack)](https://cloud-native.slack.com/archives/C03GDP0H023)

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

| Signal/Component      | Overall Status     |
| --------------------  | ------------------ |
| Logs-API              | Alpha*             |
| Logs-SDK              | Alpha              |
| Logs-OTLP Exporter    | Alpha              |
| Logs-Appender-Tracing | Alpha              |
| Metrics-API           | Alpha              |
| Metrics-SDK           | Alpha              |
| Metrics-OTLP Exporter | Alpha              |
| Traces-API            | Beta               |
| Traces-SDK            | Beta               |
| Traces-OTLP Exporter  | Beta               |

*OpenTelemetry Rust is not introducing a new end user callable Logging API.
Instead, it provides [Logs Bridge
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/bridge-api.md),
that allows one to write log appenders that can bridge existing logging
libraries to the OpenTelemetry log data model. The following log appenders are
available:

* [opentelemetry-appender-log](opentelemetry-appender-log/README.md)
* [opentelemetry-appender-tracing](opentelemetry-appender-tracing/README.md)

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
    trace::{Tracer, TracerProvider as _},
};
use opentelemetry_sdk::trace::TracerProvider;

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

The example above requires the following packages:

```toml
# Cargo.toml
[dependencies]
opentelemetry = "0.22"
opentelemetry_sdk = "0.22"
opentelemetry-stdout = { version = "0.3", features = ["trace"] }
```

See the [examples](./examples) directory for different integration patterns.

## Overview of crates

The following crates are maintained in this repo:

* [`opentelemetry`] This is the OpenTelemetry API crate, and is the crate
  required to instrument libraries and applications. It contains Context API,
  Baggage API, Propagators API, Logging Bridge API, Metrics API, and Tracing
  API.
* [`opentelemetry-sdk`] This is the OpenTelemetry SDK crate, and contains the
  official OpenTelemetry SDK implementation. It contains Logging SDK, Metrics
  SDK, and Tracing SDK. It also contains propagator implementations.
* [`opentelemetry-otlp`] - exporter to send telemetry (logs, metrics and traces)
  in the [OTLP
  format](https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/protocol)
  to an endpoint accepting OTLP. This could be the [OTel
  Collector](https://github.com/open-telemetry/opentelemetry-collector),
  telemetry backends like [Jaeger](https://www.jaegertracing.io/),
  [Prometheus](https://prometheus.io/docs/prometheus/latest/feature_flags/#otlp-receiver)
  or [vendor specific endpoints](https://opentelemetry.io/ecosystem/vendors/).
* [`opentelemetry-stdout`] exporter for sending logs, metrics and traces to
  stdout, for learning/debugging purposes.  
* [`opentelemetry-http`] This crate contains utility functions to help with
  exporting telemetry, propagation, over [`http`].
* [`opentelemetry-appender-log`] This crate provides logging appender to route
  logs emitted using the [log](https://docs.rs/log/latest/log/) crate to
  opentelemetry.
* [`opentelemetry-appender-tracing`] This crate provides logging appender to
  route logs emitted using the [tracing](https://crates.io/crates/tracing) crate
  to opentelemetry.  
* [`opentelemetry-jaeger-propagator`] provides context propagation using [jaeger
  propagation
  format](https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format).
* [`opentelemetry-prometheus`] provides a pipeline and exporter for sending
  metrics to [`Prometheus`].
* [`opentelemetry-semantic-conventions`] provides standard names and semantic
  otel conventions.
* [`opentelemetry-zipkin`] provides a pipeline and exporter for sending traces
  to [`Zipkin`].

In addition, there are several other useful crates in the [OTel Rust Contrib
repo](https://github.com/open-telemetry/opentelemetry-rust-contrib). A lot of
crates maintained outside OpenTelemetry owned repos can be found in the
[OpenTelemetry
Registry](https://opentelemetry.io/ecosystem/registry/?language=rust).

[`opentelemetry`]: https://crates.io/crates/opentelemetry
[`opentelemetry-sdk`]: https://crates.io/crates/opentelemetry-sdk
[`opentelemetry-appender-log`]: https://crates.io/crates/opentelemetry-appender-log
[`opentelemetry-appender-tracing`]: https://crates.io/crates/opentelemetry-appender-tracing
[`opentelemetry-http`]: https://crates.io/crates/opentelemetry-http
[`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
[`opentelemetry-stdout`]: https://crates.io/crates/opentelemetry-stdout
[`opentelemetry-jaeger-propagator`]: https://crates.io/crates/opentelemetry-jaeger-propagator
[`opentelemetry-prometheus`]: https://crates.io/crates/opentelemetry-prometheus
[`Prometheus`]: https://prometheus.io
[`opentelemetry-zipkin`]: https://crates.io/crates/opentelemetry-zipkin
[`Zipkin`]: https://zipkin.io
[`opentelemetry-semantic-conventions`]: https://crates.io/crates/opentelemetry-semantic-conventions
[`http`]: https://crates.io/crates/http

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

The Rust special interest group (SIG) meets weekly on Tuesdays at 9 AM Pacific
Time. The meeting is subject to change depending on contributors' availability.
Check the [OpenTelemetry community
calendar](https://github.com/open-telemetry/community?tab=readme-ov-file#calendar)
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

## Approvers and Maintainers

For GitHub groups see the [code owners](CODEOWNERS) file.

### Maintainers

* [Cijo Thomas](https://github.com/cijothomas)
* [Harold Dost](https://github.com/hdost)
* [Julian Tescher](https://github.com/jtescher)
* [Lalit Kumar Bhasin](https://github.com/lalitb)
* [Zhongyang Wu](https://github.com/TommyCpp)

### Approvers

* [Shaun Cox](https://github.com/shaun-cox)

### Emeritus

* [Dirkjan Ochtman](https://github.com/djc)
* [Jan Kühle](https://github.com/frigus02)
* [Isobel Redelmeier](https://github.com/iredelmeier)
* [Mike Goldsmith](https://github.com/MikeGoldsmith)

### Thanks to all the people who have contributed

[![contributors](https://contributors-img.web.app/image?repo=open-telemetry/opentelemetry-rust)](https://github.com/open-telemetry/opentelemetry-rust/graphs/contributors)
