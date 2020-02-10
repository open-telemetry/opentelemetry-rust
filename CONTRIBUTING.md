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

### How to Send Pull Requests

Everyone is welcome to contribute code to `opentelemetry-rust` via
GitHub pull requests (PRs).

```sh
$ git clone https://github.com/open-telemetry/opentelemetry-rust
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
