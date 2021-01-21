# Versioning

This document describes the versioning policy for this repository. This policy
is designed so the following goals can be achieved.

## Goals

### API Stability

Once the API for a given signal (spans, logs, metrics, baggage) has been
officially released, that API module will function with any SDK that has the
same major version, and equal or greater minor or patch version.

For example, libraries that are instrumented with `opentelemetry 1.0.1` will
function in applications using `opentelemetry 1.11.33` or `opentelemetry
1.3.4`.

### SDK Stability

Public portions of the SDK (constructors, configuration, end-user interfaces)
must remain backwards compatible. Internal types are allowed to break.

## Policy

* Releases will follow [SemVer](https://semver.org/).
* New telemetry signals will be introduced behind experimental 
  [cargo features](https://doc.rust-lang.org/cargo/reference/features.html).

  * New signals will be stabilized via a **minor version bump**, and are not
    allowed to break existing stable interfaces.

* GitHub releases will be made for all released versions.
* Crates will be released on crates.io

## Example Versioning Lifecycle

To better understand the implementation of the above policy, here is an example
of how the metrics and logging signals **could** stabilize.

- v1.0.0 release:
   - `opentelemetry 1.0.0`
     - Contains stable impls of trace, baggage, resource, context modules
     - experimental metrics impl behind feature flag
   - `opentelemetry-semantic-conventions 1.0.0`
     - Contains stable impls of trace, resource conventions
     - experimental metrics conventions behind feature flag
   - `opentelemetry-contrib 1.0.0`
     - Contains stable impls of 3rd party trace exporters and propagators
     - experimental metrics exporters and propagator impls behind feature flag
- v1.5.0 release (with metrics)
   - `opentelemetry 1.5.0`
     - Contains stable impls of metrics, trace, baggage, resource, context modules
     - experimental logging impl still only behind feature flag
   - `opentelemetry-semantic-conventions 1.2.0`
     - Contains stable impls of metrics, trace, resource conventions
     - experimental logging conventions still only behind feature flag
   - `opentelemetry-contrib 1.6.0`
     - Contains stable impls of 3rd party trace and metrics exporters and propagators
     - experimental logging exporters and propagator still impls behind feature flag
- v1.10.0 release (with logging)
   - `opentelemetry 1.10.0`
     - Contains stable impls of logging, metrics, trace, baggage, resource, context modules
   - `opentelemetry-semantic-conventions 1.4.0`
     - Contains stable impls of logging, metrics, trace, resource conventions
   - `opentelemetry-contrib 1.12.0`
     - Contains stable impls of 3rd party trace, metrics, and logging exporters and propagators
