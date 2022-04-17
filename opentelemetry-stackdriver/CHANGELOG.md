# Changelog

## v0.15.0

### Added

- Added mappings from OTel attributes to Google Cloud Traces #744

### Changed

- Upgrade to opentelemetry v0.18.0
- Upgrade to opentelemetry-semantic-conventions v0.10

## v0.14.0

### Changed

- Upgrade to new gcp_auth version (#722)
- Stop leaking dependency error types into public API (#722)
- Clarify type of MonitoredResource (#722)

### Fixed

- Fixed issue with futures dependency (#722)
- Don't set up logging channel if no logging is configured (#722)

## v0.13.0

### Changed

- Send export errors to global error handler (#705)
- Return `impl Future` to avoid spawning inside library (#703)
- Implement builder API to simplify configuration (#702)
- Use TLS configuration provided by tonic (#702)
- Optionally send events to Cloud Logging (#702)
- Exclude default `tonic-build` features #635
- Update `gcp_auth` dependency to `0.5.0` #639
- Include the server's message in error display #642
- Update `tonic` to 0.6 #660
- Update gcp_auth and yup-oauth2 to latest versions #700
- Update to opentelemetry v0.17.0

### Fixed

- Avoid calling log from inside exporter #709

## v0.12.0

### Changed

- Update to opentelemetry v0.16.0

## v0.11.0

### Changed

- Update to opentelemetry v0.15.0

## v0.10.0

### Changed

- Update to opentelemetry v0.14.0

## v0.9.0

### Changed
- Move opentelemetry-stackdriver into opentelemetry-rust repo #487
