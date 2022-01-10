# Changelog

## Unreleased

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.8.0...v0.9.0)

### Maturity

* Remove if no changes for this section before release.

### Changed

* Remove if no changes for this section before release.

### Added

* Remove if no changes for this section before release.

### Removed

* Remove if no changes for this section before release.

## 0.8.0 - 2021-03-23

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.7.0...v0.8.0)

### Changed: Metrics

* :stop_sign: [DEPRECATION] Deprecate IntSum, IntGauge, and IntDataPoint (#278)
* :stop_sign: [DEPRECATION] Deprecate IntExemplar (#281)
* :stop_sign: [DEPRECATION] Deprecate IntHistogram (#270)
* :stop_sign: [BREAKING] Deprecate `labels` field from NumberDataPoint, HistogramDataPoint, SummaryDataPoint and add equivalent `attributes` field (#283)
* :stop_sign: [BREAKING] Deprecate `filtered_labels` field from Exemplars and add equivalent `filtered_attributes` field (#283)
* :stop_sign: [BREAKING] Rename DoubleGauge to Gauge (#278)
* :stop_sign: [BREAKING] Rename DoubleSum to Sum (#278)
* :stop_sign: [BREAKING] Rename DoubleDataPoint to NumberDataPoint (#278)
* :stop_sign: [BREAKING] Rename DoubleSummary to Summary (#269)
* :stop_sign: [BREAKING] Rename DoubleExemplar to Exemplar (#281)
* :stop_sign: [BREAKING] Rename DoubleHistogram to Histogram (#270)
* :stop_sign: [DATA MODEL CHANGE] Make explicit bounds compatible with OM/Prometheus (#262)

## 0.7.0 - 2021-01-28

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.6.0...v0.7.0)

### Maturity

$$$Protobuf Encodings:**

* `collector/metrics/*` is now considered `Beta`. (#223)
* `collector/logs/*` is now considered `Alpha`. (#228)
* `logs/*` is now considered `Alpha`. (#228)
* `metrics/*` is now considered `Beta`. (#223)

### Changed

* Common/Logs/Metrics/Traces - Clarify empty instrumentation (#245)

### Added

* Metrics - Add SummaryDataPoint support to Metrics proto (#227)

## 0.6.0 - 2020-10-28

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.5.0...v0.6.0)

### Maturity

* Clarify maturity guarantees (#225)

### Changed

* Traces - Deprecated old Span status code and added a new status code according to specification (#224)
** Marked for removal `2021-10-22` given Stability Guarantees.
* Rename ProbabilitySampler to TraceIdRatioBased (#221)

## 0.5.0 - 2020-08-31

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.4.0...v0.5.0)

### Maturity Changes

**Protobuf Encodings:**

* `collector/trace/*` is now `Stable`.
* `common/*` is now `Stable`.
* `resource/*` is now `Stable`.
* `trace/trace.proto` is now `Stable`. (#160)

**JSON Encodings:**

* All messages are now `Alpha`.

### Changed

* :stop_sign: [BREAKING] Metrics - protocol was refactored, and lots of breaking changes.
** Removed MetricDescriptor and embedded into Metric and the new data types.
** Add new data types Gauge/Sum/Histogram.
** Make use of the "AggregationTemporality" into the data types that allow that support.
* Rename enum values to follow the proto3 style guide.

### Added

* Enable build to use docker image otel/build-protobuf to be used in CI.
** Can also be used by the languages to generate protos.

### Removed

* :stop_sign: [BREAKING] Remove generated golang structs from the repository

### Errata

The following was announced in the release, but has not yet been considered stable. Please see the latest
README.md for actual status.

> This is a Release Candidate to declare Metrics part of the protocol Stable.

## 0.4.0 - 2020-06-23

Full list of differences found in [this compare.](https://github.com/open-telemetry/opentelemetry-proto/compare/v0.3.0...v0.4.0)

### Changed

* Metrics - Add temporality to MetricDescriptor (#140).

### Added

* Metrics - Add Monotonic Types (#145)
* Common/Traces - Added support for arrays and maps for attribute values (AnyValue) (#157).

### Removed

* :stop_sign: [BREAKING] Metrics - Removed common labels from MetricDescriptor (#144).

### Errata

The following was announced in the release, but this was not considered Stable until `v0.5.0`

> This is a Release Candidate to declare Traces part of the protocol Stable.

## 0.3.0 - 2020-03-23

* Initial protos for trace, metrics, resource and OTLP.
