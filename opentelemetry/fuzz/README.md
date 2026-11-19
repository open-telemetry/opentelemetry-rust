# Fuzzing `opentelemetry`

This directory contains coverage-guided fuzz targets owned by the `opentelemetry`
crate. It is an independent Cargo project so its nightly-only tooling and
dependencies don't affect the stable root workspace or the published crate.
The targets exercise the local crate through its public API.

## Setup

```sh
rustup toolchain install nightly --component rust-src
cargo install cargo-fuzz
```

## Context targets

| Target | Checks |
| --- | --- |
| `context_stack_model` | Compares generated context creation, derivation, attachment, suppression, span-context, and out-of-order guard-drop operations with a simple public-semantics model. |

From the repository root:

```sh
cargo +nightly fuzz list --fuzz-dir opentelemetry/fuzz
cargo +nightly fuzz run --fuzz-dir opentelemetry/fuzz context_stack_model
```

Bound a local run with libFuzzer arguments after `--`:

```sh
cargo +nightly fuzz run --fuzz-dir opentelemetry/fuzz context_stack_model -- \
  -max_total_time=60 -timeout=10 -max_len=4096
```

Run the deterministic smoke tests and build every target with:

```sh
cargo +nightly test --manifest-path opentelemetry/fuzz/Cargo.toml --lib
cargo +nightly clippy --manifest-path opentelemetry/fuzz/Cargo.toml --all-targets -- -D warnings
cargo +nightly fuzz build --fuzz-dir opentelemetry/fuzz
```

libFuzzer writes failures to `opentelemetry/fuzz/artifacts/<target>/`. Reproduce
and minimise a failure with:

```sh
cargo +nightly fuzz run --fuzz-dir opentelemetry/fuzz <target> <artifact>
cargo +nightly fuzz tmin --fuzz-dir opentelemetry/fuzz <target> <artifact>
```

Turn each confirmed failure into a deterministic regression test in the owning
crate before removing its artifact.
