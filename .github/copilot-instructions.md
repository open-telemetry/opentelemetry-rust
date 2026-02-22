# OpenTelemetry Rust Copilot Instructions

This repository contains the OpenTelemetry implementation for Rust. When working with this codebase, please follow these guidelines:

## Development Tools and Commands

1. **Use clippy, rustfmt, and cargo check regularly.**
   - Run `cargo clippy --workspace --all-targets --all-features` for comprehensive linting
   - Run `cargo fmt --all` to format code consistently
   - Run `cargo check --workspace` to verify compilation

2. **For individual crate development:**
   - Use `cargo build`, `cargo test`, `cargo clippy` within specific crates for focused development
   - This is more efficient when working on features for a single crate

3. **Use the provided scripts for final validation:**
   - Run `./scripts/precommit.sh` before committing (runs format, lint, and test)
   - Run `./scripts/lint.sh` for comprehensive linting across all features
   - Run `./scripts/test.sh` for running tests
   - These scripts run in CI, so use them as final checks rather than with each change

## Workspace Structure

This is a Cargo workspace containing multiple crates:
- `opentelemetry` - Core API crate
- `opentelemetry-sdk` - SDK implementation
- `opentelemetry-otlp` - OTLP exporter
- `opentelemetry-stdout` - Stdout exporter
- And many more specialized crates

When making changes, consider the impact across the entire workspace.

## Feature Flags and Compilation

- This codebase uses extensive feature flags for signals (`traces`, `metrics`, `logs`)
- Experimental features are behind the `otel_unstable` feature flag
- Use `cargo hack --each-feature` to test across feature combinations
- Some crates like `opentelemetry-prometheus` are outside the main workspace

## Code Standards

- **MSRV**: Minimum Supported Rust Version is 1.75
- **Error Handling**: Wrap errors in signal-specific types (`TraceError`, `MetricError`, `LogsError`)
- **Commits**: Use conventional commit format (e.g., `feat:`, `fix:`, `docs:`)
- **Testing**: Run `cargo test --workspace` for comprehensive testing
- **Documentation**: Include examples in documentation and maintain README files

## OpenTelemetry Specifics

- Follow the OpenTelemetry specification
- Prioritize functionality over structural compliance with spec
- Use Rust idioms rather than forcing spec API patterns
- Configuration priority: compile-time config > Environment variables
- Use `#[cfg(feature = "otel_unstable")]` for experimental features

## Build Requirements

- For `opentelemetry-otlp`: Requires `protoc` (Protocol Buffers compiler)
- Clone with submodules: `git clone --recurse-submodule`
- Set `PROTOC` environment variable if needed: `export PROTOC=$(which protoc)`

## Testing and Validation

- Run benchmarks with `cargo bench` for performance-sensitive changes
- Test examples in the `examples/` directory to ensure they work
- Validate changes don't break existing behavior across workspace
- Consider impact on different runtime features (`rt-tokio`, etc.)