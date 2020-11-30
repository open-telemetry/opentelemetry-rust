# Contributing to opentelemetry-rust

The Rust special interest group (SIG) meets regularly. See the
OpenTelemetry
[community](https://github.com/open-telemetry/community#rust-sig)
repo for information on this and other language SIGs.

See the [public meeting
notes](https://docs.google.com/document/d/1tGKuCsSnyT2McDncVJrMgg74_z8V06riWZa0Sr79I_4/edit)
for a summary description of past meetings. To request edit access,
join the meeting or get in touch on
[Gitter](https://gitter.im/open-telemetry/opentelemetry-rust).

## Pull Requests

### Prerequisites

Crate `opentelemetry-otlp` uses gRPC. So you will need the following to build:

- [protoc](https://github.com/protocolbuffers/protobuf)
- [cmake](https://cmake.org)
- [llvm](https://releases.llvm.org/download.html) (and `LIBCLANG_PATH` environment variable pointing to the `bin` directory of LLVM install)

### How to Send Pull Requests

Everyone is welcome to contribute code to `opentelemetry-rust` via
GitHub pull requests (PRs).

```sh
$ git clone --recurse-submodule https://github.com/open-telemetry/opentelemetry-rust
```

Enter the newly created directory and add your fork as a new remote:

```sh
$ git remote add <YOUR_FORK> git@github.com:<YOUR_GITHUB_USERNAME>/opentelemetry-rust
```

Check out a new branch, make modifications, run linters and tests, and
push the branch to your fork:

```sh
$ git checkout -b <YOUR_BRANCH_NAME>
# edit files
$ git add -p
$ git commit
$ git push <YOUR_FORK> <YOUR_BRANCH_NAME>
```

Open a pull request against the main
[opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust)
repo.

### How to Receive Comments

* If the PR is not ready for review, please put `[WIP]` in the title,
  tag it as `work-in-progress`, or mark it as
  [`draft`](https://github.blog/2019-02-14-introducing-draft-pull-requests/).
* Make sure CLA is signed and CI is clear.

### How to Get PRs Merged

A PR is considered to be **ready to merge** when:

* It has received approval from Collaborators/Maintainers.
* Major feedback is resolved.

Any Collaborator/Maintainer can merge the PR once it is **ready to
merge**.

## Design Choices

As with other OpenTelemetry clients, opentelemetry-rust follows the
[opentelemetry-specification](https://github.com/open-telemetry/opentelemetry-specification).

It's especially valuable to read through the [library
guidelines](https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/library-guidelines.md).

### Focus on Capabilities, Not Structure Compliance

OpenTelemetry is an evolving specification, one where the desires and
use cases are clear, but the method to satisfy those uses cases are
not.

As such, Contributions should provide functionality and behavior that
conforms to the specification, but the interface and structure is
flexible.

It is preferable to have contributions follow the idioms of the
language rather than conform to specific API names or argument
patterns in the spec.

For a deeper discussion, see:
https://github.com/open-telemetry/opentelemetry-specification/issues/165

### Error Handling
Currently, the Opentelemetry Rust SDK has two ways to handle errors. In the situation where errors are not allowed to return. One should call global error handler to process the errors. Otherwise, one should return the errors. 

The Opentelemetry Rust SDK comes with an error type `openetelemetry::Error`. For different function, one error has been defined. All error returned by trace module MUST be wrapped in `opentelemetry::trace::TraceError`. All errors returned by metrics module MUST be wrapped in `opentelemetry::metrics::MetricsError`. 

For users that want to implement their own exporters. It's RECOMMENDED to wrap all errors from the exporter into a crate-level error type, and implement `ExporterError` trait.  

## Style Guide

* Run `cargo clippy --all` - this will catch common mistakes and improve
your Rust code
* Run `cargo fmt` - this will find and fix code formatting
issues.

## Testing and Benchmarking

* Run `cargo test --all` - this will execute code and doc tests for all
projects in this workspace.
* Run `cargo bench` - this will run benchmarks to show performance 
regressions

## Approvers and Maintainers

See the [code owners](CODEOWNERS) file.

### Become an Approver or a Maintainer

See the [community membership document in OpenTelemetry community
repo](https://github.com/open-telemetry/community/blob/master/community-membership.md).
