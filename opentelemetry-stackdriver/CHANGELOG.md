# Changelog

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
