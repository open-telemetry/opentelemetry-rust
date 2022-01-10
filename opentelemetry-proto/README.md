![OpenTelemetry — An observability framework for cloud-native software.][splash]

[splash]: https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo-text.png

# OpenTelemetry Proto
This crate contains generated files from [opentelemetry-proto](https://github.com/open-telemetry/opentelemetry-proto)
repository and transformation between types from generated files and types defined in [opentelemetry](https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry).

Based on the build tool needed, users can choose to generate files using [tonic](https://github.com/hyperium/tonic)
, [prost](https://github.com/tokio-rs/prost) or [grpcio](https://github.com/tikv/grpc-rs).

