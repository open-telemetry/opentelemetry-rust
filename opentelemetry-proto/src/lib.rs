//! This crate contains generated files from [opentelemetry-proto](https://github.com/open-telemetry/opentelemetry-proto)
//! repository and transformation between types from generated files and types defined in [opentelemetry](https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry)
//!
//! Based on the build tool needed, users can choose to generate files using [tonic](https://github.com/hyperium/tonic)
//! , [prost](https://github.com/tokio-rs/prost) or [grpcio](https://github.com/tikv/grpc-rs).

// proto mod contains file generated by protobuf or other build tools.
// we should manually change it. Thus skip format and lint check.
#[rustfmt::skip]
#[allow(warnings)]
#[doc(hidden)]
mod proto;

#[cfg(feature = "gen-protoc")]
pub use proto::grpcio;
#[cfg(feature = "gen-prost")]
pub use proto::prost;
#[cfg(feature = "gen-tonic")]
pub use proto::tonic;

mod transform;
