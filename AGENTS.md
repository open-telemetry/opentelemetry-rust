# Agent Instructions

Guidance for AI coding agents (GitHub Copilot, Claude, Codex, Cursor, etc.) working in
this repository. Human contributors should follow [CONTRIBUTING.md](CONTRIBUTING.md);
this file captures the conventions an automated agent must respect to keep CI
green and reviews short.

## Repository Layout

This is a Cargo workspace. Each top-level `opentelemetry-*` directory is a
separately published crate. `opentelemetry-prometheus` is intentionally
**outside** the workspace and must be linted/tested via its own manifest.
See [docs/design/architecture.md](docs/design/architecture.md) for the
API/SDK split and crate responsibilities.

## Required Pre-Push Checks

Always run these before pushing. They're cheap and catch the lint failures CI
will otherwise flag.

```sh
# 1. Format every file in the workspace (always).
cargo fmt --all

# 2. Clippy on the crates you actually touched, with -Dwarnings.
#    Replace <crate> with the directory name (e.g. opentelemetry-sdk).
cargo clippy -p <crate> --all-targets --all-features -- -Dwarnings

# 3. Tests for the crates you touched.
cargo test -p <crate> --all-features --lib
```

For changes inside `opentelemetry-prometheus` (non-workspace), run from that
directory:

```sh
(cd opentelemetry-prometheus && cargo clippy --all-targets --all-features -- -Dwarnings)
(cd opentelemetry-prometheus && cargo test --all-features --lib)
```

### When to Run the Full Precommit

The full [scripts/precommit.sh](scripts/precommit.sh) runs `cargo update`,
workspace-wide clippy, the full `cargo hack --each-feature` matrix, and the
entire test suite. It takes a long time. Reserve it for:

- Changes touching multiple crates or shared types (e.g. anything in
  `opentelemetry/src/`).
- Public API changes, trait signature changes, or feature-flag changes.
- Dependency bumps in `Cargo.toml`.
- Changes to build scripts, proto definitions, or codegen.
- Anything you suspect could regress feature-gated builds.

For small, single-crate changes (a bug fix, a doc tweak, a benchmark, a test),
the targeted commands above are sufficient — CI will catch anything missed.

## Formatting and Lint Rules

- `rustfmt` is enforced (`cargo fmt --all -- --check` runs in CI). Always
  format before committing; do not hand-format.
- Clippy runs with `-Dwarnings` everywhere. A new warning fails CI.
- Do not introduce `#[allow(...)]` to silence clippy without a comment
  explaining why.

## Commit and PR Conventions

- PR titles follow [Conventional Commits](https://www.conventionalcommits.org/)
  (e.g. `fix(sdk): ...`, `feat(otlp): ...`, `docs: ...`). The
  `validate-pr-title` check enforces this.
- Keep PRs focused and under ~500 lines where practical (see
  [CONTRIBUTING.md](CONTRIBUTING.md#pull-request-size-and-scope)).
- Refactors must be in their own PR with no behavior changes.
- Update the relevant crate's `CHANGELOG.md` under the `## vNext` /
  `## Unreleased` section for any user-visible change.
- Do not commit generated files, editor settings, or unrelated whitespace
  changes.

## Things Not to Do

- Do not push directly to `open-telemetry/opentelemetry-rust`. Push to a
  fork and open a PR.
- Do not bypass git hooks or CI checks (`--no-verify`, `[skip ci]`, etc.).
- Do not hand-edit `Cargo.lock`; let `cargo` manage it.
- Do not add new public API without updating the corresponding
  `allowed-external-types.toml` if one exists for that crate.
- Do not create a markdown file to "document the changes you made" unless
  the user asked for it.

## Useful References

- [CONTRIBUTING.md](CONTRIBUTING.md) — full contributor guide
- [docs/design/architecture.md](docs/design/architecture.md) — workspace and
  crate layout
- [docs/adr/](docs/adr/) — past design decisions
- [scripts/precommit.sh](scripts/precommit.sh) — full local CI mirror
- [scripts/lint.sh](scripts/lint.sh) — exact clippy invocations CI runs
- [scripts/test.sh](scripts/test.sh) — exact test invocations CI runs
- [OpenTelemetry Specification](https://github.com/open-telemetry/opentelemetry-specification)
  — the source of truth for behavior across all OTel SDKs. Verify any
  spec-driven change against the relevant section.
