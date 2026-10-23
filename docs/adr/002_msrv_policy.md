# MSRV (Minimum Supported Rust Version) Policy

## Date

22 Oct 2026

## Summary

This ADR proposes replacing the current "N-3" version-count MSRV policy with a
12-month trailing window for the SDK and implementation components. Their MSRV
may be raised to any stable Rust release whose release date is at least 12
months prior to the date the MSRV bump PR is opened.

API and SDK-independent components retain their existing MSRV for as long as
practical. MSRV bumps are not considered semver-breaking changes.

## Motivation

The current MSRV policy states that "the current stable Rust compiler and the
three most recent minor versions before it will always be supported." Given
Rust's 6-week release cadence, N-3 equates to roughly 4.5 months — but the
actual MSRV (1.75, released December 2023) is nearly three years old as of
this writing. The stated policy and practice are misaligned.

A time-based policy addresses this in a few ways:

- It is easier to reason about. Users and maintainers can determine eligibility
  from a release date without needing to know Rust's release cadence or the
  current stable version number.
- It is more predictable for downstream consumers. A 12-month window gives
  library authors and end users a full year to upgrade their toolchain before
  they are affected.
- It aligns with the broader Rust ecosystem. The most widely-adopted formal
  MSRV policies in the Rust ecosystem are time-based (see
  [Ecosystem Context](#ecosystem-context) below).
- It is consistent with OpenTelemetry cross-language guidance. The OpenTelemetry
  specification states: "Each language implementation SHOULD define how the
  removal of a supported language/runtime version affects its versioning. As a
  rule of thumb, it SHOULD follow the conventions in the given ecosystem."
  ([source](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/versioning-and-stability.md#language-version-support))

## Policy

### The rule

The MSRV of the SDK and implementation components may be raised to any stable
Rust version whose release date is at least 12 months prior to the date the
MSRV bump PR is opened. API and SDK-independent components should retain their
existing MSRV for as long as practical.

### Scope

API and SDK-independent components are published crates that do not have a
runtime dependency on `opentelemetry_sdk`, such as the `opentelemetry` API and
`opentelemetry-propagator-b3` crates. These crates may be used by libraries
that expose OpenTelemetry types without selecting or configuring an SDK.

The SDK implementation group includes `opentelemetry_sdk` and published crates
with a runtime dependency on it, such as `opentelemetry-otlp` and
`opentelemetry-stdout`. These crates may use a higher MSRV.

### Semver treatment

MSRV bumps are not semver-breaking changes. They are shipped in minor releases
only, never in patch releases.

The `rust-version` field in each crate's `Cargo.toml` is the source of truth.
These values may differ between API and SDK-independent components and SDK
implementation components.

### CI enforcement

CI must verify compilation against the declared MSRV on every PR. The MSRV CI
job should use a pinned toolchain (e.g. `+1.85.0`) and run at minimum
`cargo check --all-features` against all published crates.

### Process for bumping the MSRV

1. Open a PR that updates the `rust-version` field in the affected crates and
   any crates that inherit the higher requirement through runtime dependencies.
2. For an SDK or implementation component, verify that the target Rust version
   was released at least 12 months ago. For an API or SDK-independent
   component, explain why retaining its existing MSRV is no longer practical.
3. The PR description should note what the bump enables (dependency removal,
   new std APIs, edition upgrade, etc.). This context is informational and
   helps reviewers — it is not a gate.
4. Standard review and approval process applies.

### When to bump

MSRV bumps should be motivated by concrete downstream benefit, not merely
because the trailing window permits it. Examples of valid motivations include:

- A dependency release that the project needs is only available at a higher
  MSRV (upstream deduplication benefit for all downstreams).
- A `std` API replaces an external dependency, reducing the transitive
  dependency tree for all consumers.
- A new Rust edition brings safety or correctness improvements relevant to
  the project (e.g. stricter `unsafe` defaults).
- A `core`/`alloc` API enables meaningful `no_std` support improvements.

The 12-month window defines what is *permissible*, not what is *expected*.
Maintainers should exercise judgment and prefer the lowest MSRV that
satisfies the project's current needs.

The threshold for raising an API or SDK-independent component should be higher
because of its broader downstream use. Workspace uniformity alone is not a
reason to raise its MSRV.

## Practical impact

As of October 2026, this policy would permit raising the SDK MSRV as high as
1.90 (September 2025) while API and SDK-independent components such as
`opentelemetry` and `opentelemetry-propagator-b3` remained on Rust 1.75. Crates
with a runtime dependency on the SDK would inherit the higher requirement.
Some capabilities available at various MSRV levels include:

- **1.80+**: `std::sync::LazyLock` — could replace `once_cell` / `lazy_static`
  for internal static initialization.
- **1.81+**: `#[expect(lint)]` — works like `#[allow]` but warns when the
  suppressed lint stops firing, preventing stale suppressions from
  accumulating.
- **1.81+**: `core::error::Error` — the `Error` trait in `core` strengthens
  the `no_std` story and may benefit downstream library consumers.
- **1.85+**: Edition 2024 eligibility — brings stricter `unsafe_op_in_unsafe_fn`
  defaults and improved lifetime capture rules.

Any actual MSRV bump would be proposed and evaluated separately based on
concrete need at the time.

For reference, here is what the 12-month window looks like over time:

| Date | Current Stable | 12-Month Window Allows |
|------|---------------|------------------------|
| Oct 2026 (today) | 1.99 | 1.90 (Sep 2025) |
| Apr 2027 | ~1.103 | 1.95 (Apr 2026) |
| Oct 2027 | ~1.108 | 1.99 (Oct 2026) |

## Ecosystem context

### Rust ecosystem

The most widely-used Rust crates with formal, documented MSRV policies use a
time-based or version-count approach. The MSRVs below are declared by each
crate's latest stable published release as of 22 October 2026:

| Project | Documented Policy | Window | Latest Published MSRV |
|---------|-------------------|--------|-----------------------|
| [tokio](https://github.com/tokio-rs/tokio#supported-rust-versions) | "The new Rust version must have been released at least six months ago" | 6 months | 1.85 (~20 months) |
| [hyper](https://github.com/hyperium/hyper/blob/master/docs/MSRV.md) | "A compiler version released within the last 6 months can compile hyper" | 6 months | 1.63 (>4 years) |
| [tower](https://github.com/tower-rs/tower#supported-rust-versions) | "The new Rust version must have been released at least six months ago" | 6 months | 1.64 (~4 years) |
| [tracing](https://github.com/tokio-rs/tracing#supported-rust-versions) | "The current stable Rust compiler and the three most recent minor versions before it will always be supported" | N-3 | 1.65 (~4 years) |

In practice, even projects with aggressive stated policies bump conservatively
— their latest published MSRVs are often years behind what the policy permits.
Our 12-month window is twice as conservative as the 6-month policies above.
The [when to bump](#when-to-bump) guidelines ensure the window functions as a
ceiling rather than a schedule.

### OpenTelemetry language SDKs

Other OpenTelemetry language SDKs handle minimum version support as follows:

| Language | Minimum | Policy |
|----------|---------|--------|
| [Go](https://github.com/open-telemetry/opentelemetry-go#compatibility) | Go 1.26 | Follows upstream Go support: "each major Go release is supported until there are two newer major releases" |
| [Python](https://github.com/open-telemetry/opentelemetry-python#python-version-support) | Python 3.10 | "We remove support for old Python versions 6 months after they reach their end of life" |
| [Node.js](https://github.com/open-telemetry/opentelemetry-js#supported-runtimes) | Node.js 22 | "Only Node.js Active or Maintenance LTS versions are supported" |
| [.NET](https://github.com/open-telemetry/opentelemetry-dotnet#supported-net-versions) | net462 | "All the officially supported versions of .NET and .NET Framework (except .NET Framework 3.5)" |
| [Java](https://github.com/open-telemetry/opentelemetry-java/blob/main/VERSIONING.md#language-version-compatibility) | Java 8 | "Changing requires major version bump" |

The common pattern across OTel SDKs is to follow the conventions of each
language's ecosystem. Rust does not have a concept of upstream EOL for compiler
versions, so a time-based trailing window is the closest analog to the
lifecycle-based policies used by Go, Python, and Node.js.

## Alternatives considered

### 6-month trailing window

This is the de facto standard among the most widely-used Rust crates (tokio,
hyper, tower). It would allow faster adoption of new language features.

We did not go with this because opentelemetry crates are infrastructure —
depended on by downstream libraries and instrumentation across the ecosystem.
A 6-month window, while common for application-oriented crates, is more
aggressive than appropriate for a project whose MSRV bumps ripple outward
through transitive dependencies.

### N-3 version-count (current policy)

The existing documented policy supports the current stable Rust compiler and
the three most recent minor versions before it. This is equivalent to roughly
4.5 months given Rust's 6-week release cadence.

There are two problems with keeping this policy:

1. On paper, N-3 allows bumping to a version that may be only ~4.5 months
   old — more aggressive than 6-month policies. In practice, the project has
   been far more conservative, which is exactly the policy-vs-practice
   disconnect this ADR aims to resolve.
2. The current MSRV (1.75) is nearly three years old while the N-3 policy would
   currently allow up to 1.96 (with stable at 1.99). A documented policy that
   diverges this far from practice provides little value as a guide for
   contributors or consumers.

A 12-month window is genuinely more conservative than both alternatives while
still allowing the project to benefit from language improvements within a
reasonable timeframe.

## References

- [PR #3352 — Bump MSRV to 1.85](https://github.com/open-telemetry/opentelemetry-rust/pull/3352) — the discussion that motivated this ADR
- [OpenTelemetry specification — Language version support](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/versioning-and-stability.md#language-version-support)
- [Rust RFC 2495 — `rust-version` in Cargo.toml](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
- [Rust RFC 3537 — MSRV-aware resolver](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html)
- [Rust release history](https://releases.rs/)
