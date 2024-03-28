# Releasing OpenTelemetry Rust

The primary audience for this is the SIG Maintainers. It provides the list of steps for how to release the crates and the
considerations to make before releasing the crate. It may provide use to consumers of the crate if/when we develop a
release cadence.

## Release cadence

As of Feb 2024, there is no established cadence for the OpenTelemetry crates. The balance is required between too many
breaking changes in a single release, and since we have instability flipping between implementations across 0.x
releases. Perhaps once we've established a 1.0 we can consider adopting a consistent release cadence.

## Considerations

A draft PR can be created, but before releasing consider the following:

* Are there any pending pull requests which should be included in the next release?
  * Are they blockers?
* Are there any unresolved issues which should be resolved before the next release?
* Bring it up at a SIG meeting, this can usually get some of these questions answered sooner than later. It will also
  help establish a person to perform the release. Ideally this can be someone different each time to ensure that the
  process is documented.

## Steps to Release

1. Create a release PR

* For each crate
  * Bump appropriate version
  * Update change logs to reflect release version.
  * Update API/SDK version as necessary
  * Attach `integration test` label to the PR to run integration tests
* If there's a large enough set of changes, consider writing a migration guide.

2. Merge the PR

* Get reviews from other Maintainers
* Ensure that there haven't been any interfering PRs

3. Tag the release commit based on the [tagging convention](#tagging-convention). It should usually be a bump on minor version before 1.0
4. Create Github Release
5. [Publish](#publishing-crates) to crates.io using the version as of the release commit
6. Post to [#otel-rust](https://cloud-native.slack.com/archives/C03GDP0H023) on CNCF Slack.

## Tagging Convention

For each crate: it should be `<crate-name>-<version>` `<version>` being the simple `X.Y.Z`.
For example:

```sh
git tag -a opentelemetry-http-0.11.1 -m "opentelemetry-http 0.11.1 release"
git push origin opentelemetry-http-0.11.1
```

## Publishing Crates

For now we use the [basic procedure](https://doc.rust-lang.org/cargo/reference/publishing.html) from crates.io.

Follow this for each crate as necessary.

For any new crates remember to add open-telemetry/rust-maintainers to the list.
