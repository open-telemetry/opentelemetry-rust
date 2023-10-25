![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry user_events Exporter

## Overview

[user_events](https://docs.kernel.org/trace/user_events.html) is Linux solution for user process tracing, similar to ETW (Event Tracing for Windows) on Windows. It builds on top of the Linux Tracepoints, and so allows user processes to create events and trace data that can be viewed via existing tools like ftrace and perf.

This kernel feature is supported started in Linux kernel 5.18 onwards. The feature enables
 - A faster path for tracing from user mode application utilizing kernel mode memory address space. 
 - User processes can now export telemetry events only when it is useful i.e, when the registered set of tracepoint events are enabled.

 This user_events exporter enables applications to use OpenTelemetry API to capture the telemetry events, and write to user_events subsystem. From user_events, the events can be
  - Captured by the agents running locally, and listening for specific events withing user_events subsystem.
  - Or real-time monitoring using local Linux tool like [perf](https://perf.wiki.kernel.org/index.php/Main_Page) or ftrace.
