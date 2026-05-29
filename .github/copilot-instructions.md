# OpenTelemetry Rust
OpenTelemetry Rust is a Rust implementation of the OpenTelemetry observability framework for generating, collecting, and exporting telemetry data (metrics, logs, and traces).

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Initial Setup and Dependencies
Install the following required dependencies before building:
- `sudo apt-get update && sudo apt-get install -y protobuf-compiler` - Protocol Buffer compiler (required for OTLP)
- `rustup component add rustfmt clippy` - Rust formatting and linting tools
- `cargo install cargo-hack` - Tool for testing feature combinations (takes ~50s to install)
- `cargo install cargo-msrv` - Tool for verifying minimum supported Rust version (takes ~5m to install)

### Core Build and Test Commands
- `cargo check --workspace` - Basic workspace check: **1m 15s. NEVER CANCEL. Set timeout to 90+ seconds.**
- `./scripts/test.sh` - Full test suite: **4 minutes. NEVER CANCEL. Set timeout to 480+ seconds.**
- `./scripts/lint.sh` - Complete linting with all feature combinations: **6 minutes. NEVER CANCEL. Set timeout to 480+ seconds.**
- `./scripts/integration_tests.sh` - Integration tests: **3 minutes. NEVER CANCEL. Set timeout to 300+ seconds.**
- `./scripts/precommit.sh` - Complete validation (update, format, lint, test): **varies based on changes. NEVER CANCEL. Set timeout to 600+ seconds.**
- `cargo fmt --all -- --check` - Check code formatting: **<1 second**
- `cargo doc --no-deps --all-features` - Generate documentation: **20 seconds**

### Before Committing Changes
ALWAYS run these commands before committing to ensure CI will pass:
1. `cargo fmt --all` - Format all code
2. `./scripts/lint.sh` - Run complete linting (6 minutes)
3. `./scripts/test.sh` - Run all tests (4 minutes)

Alternatively, run the complete validation with:
- `./scripts/precommit.sh` - Runs cargo update, fmt, lint, and test in sequence

## Project Structure

### Key Crates
The repository contains these main crates:
- `opentelemetry/` - Core OpenTelemetry API (Context, Baggage, Propagators, Metrics, Traces, Logs APIs)
- `opentelemetry-sdk/` - Official OpenTelemetry SDK implementation
- `opentelemetry-otlp/` - OTLP exporter for sending telemetry via OTLP protocol
- `opentelemetry-stdout/` - Stdout exporter for debugging and learning
- `opentelemetry-http/` - HTTP utilities for telemetry and propagation
- `opentelemetry-appender-log/` - Bridge for `log` crate to OpenTelemetry
- `opentelemetry-appender-tracing/` - Bridge for `tracing` crate to OpenTelemetry
- `opentelemetry-jaeger-propagator/` - Jaeger propagation format support
- `opentelemetry-prometheus/` - Prometheus metrics exporter (separate workspace member)
- `opentelemetry-semantic-conventions/` - Standard semantic conventions
- `opentelemetry-zipkin/` - Zipkin trace exporter

### Important Directories
- `scripts/` - Build, test, and validation scripts
- `examples/` - Working examples for logs, metrics, and tracing
- `docs/` - Architecture Decision Records and release notes
- `.github/workflows/` - CI/CD pipeline definitions

### Examples Directory
Test any changes by running the examples:
- `cd examples/metrics-basic && cargo run` - Basic metrics example (6 seconds)
- `cd examples/logs-basic && cargo run` - Basic logs example (4 seconds) 
- `cd examples/metrics-advanced && cargo run` - Advanced metrics with views
- `cd examples/tracing-grpc && cargo run` - gRPC tracing example
- `cd examples/tracing-http-propagator && cargo run` - HTTP tracing example

## Validation Scenarios

### After Making Changes
1. **Build Check**: Run `cargo check --workspace` to verify basic compilation
2. **Format**: Run `cargo fmt --all` to format code
3. **Quick Example Test**: Run one relevant example from `examples/` directory
4. **Full Validation**: Run `./scripts/precommit.sh` for complete validation

### Before Opening a PR
1. **Complete Test Suite**: `./scripts/test.sh` (4 minutes)
2. **Complete Linting**: `./scripts/lint.sh` (6 minutes)  
3. **Integration Tests**: `./scripts/integration_tests.sh` (3 minutes)
4. **Documentation**: `cargo doc --no-deps --all-features` (20 seconds)

### For Feature Changes
- Run examples that exercise your changes
- Test with different feature combinations using cargo-hack
- Verify MSRV compatibility with `./scripts/msrv.sh` (3 minutes)

## Common Tasks

### Rust Version Support
- **Minimum Supported Rust Version (MSRV)**: 1.75.0
- **Current stable**: Works with latest stable Rust
- Check MSRV compliance: `./scripts/msrv.sh` (3 minutes)

### Protocol Buffers
- **protoc version required**: 3.15+ (3.21.12 verified working)
- Required for `opentelemetry-otlp` crate compilation
- Install with: `sudo apt-get install protobuf-compiler`
- Some crates auto-download protoc if `PROTOC` env var not set

### Feature Flags and Testing
- Use `cargo hack --each-feature` to test all feature combinations (included in lint script)
- Key feature flags: `trace`, `metrics`, `logs`, `grpc-tonic`, `http-proto`, `reqwest-client`
- Test with `--all-features` for full functionality testing
- Test with `--no-default-features` for minimal builds

### Development Workflow
1. Make code changes
2. Run `cargo fmt --all` to format
3. Run `cargo check --workspace` for quick validation (1m 15s)
4. Test relevant examples to verify functionality
5. Run `./scripts/precommit.sh` for full validation before committing

### CI Pipeline Expectations
The CI pipeline runs:
- Tests on Ubuntu, Windows, macOS
- Rust stable and beta versions
- All feature combinations via cargo-hack
- MSRV verification
- Documentation generation
- External types checking
- Security audits with cargo-deny

### Known Issues
- Integration test `metrictests::counter_tokio_current` may fail intermittently (existing issue)
- Some dependencies require newer Rust versions but are locked to MSRV-compatible versions
- Build times can be significant due to the large number of feature combinations tested

### Performance Notes
- Initial builds are slow due to many dependencies
- Subsequent builds are much faster due to caching
- Lint script tests many feature combinations, hence the 6-minute duration
- Use `cargo check` for faster iteration during development

## Repository Information
- **License**: Apache License 2.0
- **Workspace**: Uses Cargo workspace with 13+ member crates
- **Dependencies**: Managed at workspace level in root `Cargo.toml`
- **Non-workspace member**: `opentelemetry-prometheus` (legacy reasons)