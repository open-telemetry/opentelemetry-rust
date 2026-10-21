# MSRV (Minimum Supported Rust Version) Policy

## Date

20 Feb 2026

## Summary

This ADR proposes replacing the current "N-3" version-count MSRV policy with a
12-month trailing window. Under this policy, the MSRV may be raised to any
stable Rust release whose release date is at least 12 months prior to the date
the MSRV bump PR is opened. This applies uniformly across all published
workspace crates. MSRV bumps are not considered semver-breaking changes.

## Motivation

The current MSRV policy states that "the current stable Rust compiler and the
three most recent minor versions before it will always be supported." Given
Rust's 6-week release cadence, N-3 equates to roughly 4.5 months — but the
actual MSRV (1.75, released December 2023) is over two years old as of this
writing. The stated policy and practice are misaligned.

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

The MSRV may be raised to any stable Rust version whose release date is at
least 12 months prior to the date the MSRV bump PR is opened.

### Scope

A single, uniform MSRV applies to all published crates in the workspace.
Examples, stress tests, and internal test crates are excluded.

### Semver treatment

MSRV bumps are not semver-breaking changes. They are shipped in minor releases
only, never in patch releases.

The `rust-version` field in each crate's `Cargo.toml` is the source of truth
and must be kept in sync across the workspace.

### CI enforcement

CI must verify compilation against the declared MSRV on every PR. The MSRV CI
job should use a pinned toolchain (e.g. `+1.85.0`) and run at minimum
`cargo check --all-features` against all published crates.

### Process for bumping the MSRV

1. Open a PR that updates the `rust-version` field in all published workspace
   `Cargo.toml` files.
2. Verify that the target Rust version was released at least 12 months ago.
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

## Practical impact

As an example, if this policy were adopted today, the 12-month window would
permit raising the MSRV up to 1.85 (February 2025). Some capabilities that
would become available at various MSRV levels include:

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
| Feb 2026 (today) | 1.93 | 1.85 (Feb 2025) |
| Aug 2026 | ~1.98 | 1.89 (Aug 2025) |
| Feb 2027 | ~1.102 | 1.93 (Jan 2026) |

## Ecosystem context

### Rust ecosystem

The most widely-used Rust crates with formal, documented MSRV policies use a
time-based or version-count approach:

| Project | Documented Policy | Window | Actual MSRV |
|---------|-------------------|--------|-------------|
| [tokio](https://github.com/tokio-rs/tokio#supported-rust-versions) | "The new Rust version must have been released at least six months ago" | 6 months | 1.71 (~2 years) |
| [hyper](https://github.com/hyperium/hyper/blob/master/docs/MSRV.md) | "A compiler version released within the last 6 months can compile hyper" | 6 months | 1.63 (~3 years) |
| [tower](https://github.com/tower-rs/tower#supported-rust-versions) | "The new Rust version must have been released at least six months ago" | 6 months | 1.64 (~3 years) |
| [tracing](https://github.com/tokio-rs/tracing#supported-rust-versions) | "The current stable Rust compiler and the three most recent minor versions before it will always be supported" | N-3 | — |

In practice, even projects with aggressive stated policies bump conservatively
— their actual MSRVs are often years behind what the policy permits. Our
12-month window is twice as conservative as the 6-month policies above, and the
[when to bump](#when-to-bump) guidelines ensure the window functions as a
ceiling rather than a schedule.

### OpenTelemetry language SDKs

Other OpenTelemetry language SDKs handle minimum version support as follows:

| Language | Minimum | Policy |
|----------|---------|--------|
| [Go](https://github.com/open-telemetry/opentelemetry-go#compatibility) | Go 1.24 | Follows upstream Go support: "each major Go release is supported until there are two newer major releases" |
| [Python](https://github.com/open-telemetry/opentelemetry-python#python-version-support) | Python 3.9 | "We remove support for old Python versions 6 months after they reach their end of life" |
| [Node.js](https://github.com/open-telemetry/opentelemetry-js#supported-runtimes) | Node.js 18 | "Only Node.js Active or Maintenance LTS versions are supported" |
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
2. The current MSRV (1.75) is over two years old while the N-3 policy would
   currently allow up to 1.90 (with stable at 1.93). A documented policy that
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
