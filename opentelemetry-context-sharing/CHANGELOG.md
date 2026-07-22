# Changelog

## vNext

- **Feature**: Add process context sharing support ([OTEP-4719](https://github.com/open-telemetry/oteps/pull/4719)). Publishes SDK resource attributes via a named memory mapping on Linux, enabling external readers such as the OpenTelemetry eBPF Profiler to discover process metadata.
