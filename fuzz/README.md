# Fuzzing

Coverage guided fuzz targets for OpenTelemetry Rust, built with
[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) and libFuzzer.

This crate is **not** part of the root Cargo workspace (it declares its own
`[workspace]`) because it needs a nightly toolchain. Workspace wide commands
don't build it; run everything from this directory.

## Setup

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Targets

| Target | Input | Checks |
| --- | --- | --- |
| `tracestate_parse_raw` | Arbitrary UTF-8 strings | `TraceState::from_str` agrees with the grammar oracle, parsed states hold the invariants and roundtrip through `header()` |
| `tracestate_differential` | `tracestate` headers generated from the grammar (valid, with long keys/values, or mutated) | Same as above |
| `trace_context_extract` | Generated `traceparent` and several `tracestate` fields | `TraceContextPropagator` agrees with both oracles, and inject → extract roundtrips |

## Running

```sh
cd fuzz
cargo +nightly fuzz list
cargo +nightly fuzz run tracestate_differential
# Stop after a minute.
cargo +nightly fuzz run tracestate_differential -- -max_total_time=60
```

When a target fails, libFuzzer writes the input to
`artifacts/<target>/crash-*`. To reproduce, minimize and inspect it:

```sh
cargo +nightly fuzz run <target> artifacts/<target>/crash-<hash>
cargo +nightly fuzz tmin <target> artifacts/<target>/crash-<hash>
cargo +nightly fuzz fmt <target> artifacts/<target>/minimized-from-<hash>
```

Turn every fixed crash into a unit test in the affected crate, so the
regression is caught by the regular test suite.

To see which code the corpus reaches:

```sh
cargo +nightly fuzz coverage <target>
```

`corpus/`, `artifacts/` and `coverage/` are gitignored.

## How inputs are generated

- **Generators.** [`src/generators.rs`](src/generators.rs) has hand-written
  `Arbitrary` types that build headers shaped like the W3C grammar, reading
  every decision (member count, key/value chars, ids, version) from the fuzzer
  input. Repetition counts favor short values but regularly go just past the
  spec limits. The `*Mutated` types apply a few random char edits to a
  generated header, producing inputs close to the grammar to reach error
  paths.
- **Oracle.** The headers are described as [pest](https://pest.rs) grammars in
  [`src/grammars/`](src/grammars). [`src/oracle.rs`](src/oracle.rs) derives
  parsers from them with `pest_derive`, and the `expected_*` functions combine
  those parsers with the limits a grammar can't express (at most 32 list
  members, unique keys, key/value length, non-zero ids) to compute what a spec
  compliant implementation must return. The targets compare this with the real
  implementation.

The unit tests check that the non-mutated generators only produce strings the
grammars match:

```sh
cargo +nightly test --lib
```

## Adding a target

1. Add a generator for the input to `src/generators.rs`. If the target compares
   against a spec, add a grammar to `src/grammars/` and an oracle to
   `src/oracle.rs`.
2. Add `fuzz_targets/<name>.rs` and a matching `[[bin]]` entry in `Cargo.toml`.
3. Before sending a PR, run:

```sh
cargo fmt
cargo +nightly clippy --all-targets -- -Dwarnings
cargo +nightly test --lib
cargo +nightly fuzz build
```
