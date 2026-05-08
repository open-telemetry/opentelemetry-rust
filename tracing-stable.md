# Tracing API Stabilization — Breaking Changes

Tracked items from the [Tracing API Stable](https://github.com/open-telemetry/opentelemetry-rust/milestone/1) milestone that require breaking changes to the `opentelemetry` crate before declaring stable.

> **Note on Span Events:** The OTel spec is deprecating Span Events in favor of log-based events
> ([OTEP #4430](https://github.com/open-telemetry/opentelemetry-specification/commit/b42c863), [OTEP 265](https://github.com/open-telemetry/oteps/pull/265)).
> We avoid investing in redesigning the `Event` type and scope storage/design improvements to `Link` only.

## 1. Remove `dropped_attributes_count` from API `Event` and `Link` ([#2752](https://github.com/open-telemetry/opentelemetry-rust/issues/2752))

The `opentelemetry` crate exposes `dropped_attributes_count` on `Event` and `Link`. Enforcing limits is an SDK concern. Remove from both API types; move to the SDK.

## 2. Redesign `SpanBuilder` to accept borrowed attributes ([#1109](https://github.com/open-telemetry/opentelemetry-rust/issues/1109))

`SpanBuilder::with_attributes` takes ownership via `IntoIterator<Item = KeyValue>` and allocates into a map. When sampling drops 99%+ of spans, this allocation is wasted. The API should accept `&[KeyValue]` (or similar borrowed form) and only clone to owned storage when the span is actually recorded.

## 3. Connect `Tracer` and `SpanBuilder` ([#2742](https://github.com/open-telemetry/opentelemetry-rust/issues/2742))

`SpanBuilder` is a standalone struct with no link back to its `Tracer`. It should carry a reference to the tracer that created it, changing the span-creation API surface.

## 4. `SpanId` / `TraceId` — use `NonZero` types and `Option` ([#1115](https://github.com/open-telemetry/opentelemetry-rust/issues/1115))

Replace `SpanId(u64)` / `TraceId(u128)` with `NonZeroU64` / `NonZeroU128` internally, and use `Option<SpanId>` instead of `SpanId::INVALID`. This is more idiomatic Rust and has the same memory layout thanks to niche optimization. Breaks anything matching on `SpanId::INVALID` or `TraceId::INVALID`.

## 5. Move `Link` attribute storage out of the API ([#2751](https://github.com/open-telemetry/opentelemetry-rust/issues/2751))

`Link` stores `attributes: Vec<KeyValue>` — the API dictates the data structure. Following the Logs pattern (trait + SDK-chosen storage), `Link` should carry only `SpanContext`; the SDK decides how to store link attributes. `Event` is left as-is given its deprecation path.

## 6. Audit and hide external types from public API ([#1304](https://github.com/open-telemetry/opentelemetry-rust/issues/1304))

Pre-1.0 external crate types exposed in the public API (e.g. types from `indexmap`, `ordermap`) must be wrapped or hidden behind our own types. An external dependency bump would otherwise be a semver-breaking change for us.

## 7. Feature-flag naming convention ([#1411](https://github.com/open-telemetry/opentelemetry-rust/issues/1411))

Feature flags (`experimental_*`, `spec_unstable_*`, etc.) need a consistent naming convention. Renaming existing flags is a breaking change for any `Cargo.toml` that references them.

## 8. Graduate stable-spec features out of experimental flags ([#3376](https://github.com/open-telemetry/opentelemetry-rust/issues/3376))

Features gated behind `experimental_*` / `spec_unstable_*` flags that correspond to stable specification features need to be graduated (always-on or default). Users currently enabling those flags explicitly will need to update their `Cargo.toml`.

## 9. Optimize non-recording span creation path ([#2800](https://github.com/open-telemetry/opentelemetry-rust/issues/2800))

Creating a span when the sampler returns DROP should be near-zero cost. Achieving this likely requires changes to the `Span` trait or `Tracer::build_with_context` signatures to enable a fast no-op path.
