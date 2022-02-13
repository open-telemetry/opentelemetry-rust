#[cfg(feature = "gen-tonic")]
/// Generated files using [`tonic`](https://docs.rs/crate/grpcio) and [`prost`](https://docs.rs/crate/protobuf/latest)
pub mod tonic {
    /// Service stub and clients
    pub mod collector {
        #[cfg(feature = "logs")]
        pub mod logs {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.logs.v1");
            }
        }

        #[cfg(feature = "metrics")]
        pub mod metrics {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.metrics.v1");
            }
        }

        #[cfg(feature = "traces")]
        pub mod trace {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.trace.v1");
            }
        }
    }

    /// Common types used across all signals
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.common.v1");
        }
    }

    #[cfg(feature = "logs")]
    /// Generated types used in logging.
    pub mod logs {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.logs.v1");
        }
    }

    #[cfg(feature = "metrics")]
    /// Generated types used in metrics.
    pub mod metrics {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.metrics.v1");
        }
    }

    /// Generated types used in resources.
    pub mod resource {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.resource.v1");
        }
    }

    #[cfg(feature = "traces")]
    /// Generated types used in traces.
    pub mod trace {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.trace.v1");
        }
    }

    pub use crate::transform::common::tonic::Attributes;

    #[cfg(feature = "metrics")]
    pub use crate::transform::metrics::tonic::FromNumber;
}

#[cfg(feature = "gen-protoc")]
/// Generated files using [`grpcio`](https://docs.rs/crate/grpcio) and [`protobuf`](https://docs.rs/crate/protobuf/latest)
pub mod grpcio {
    pub mod common;
    #[cfg(feature = "metrics")]
    pub mod metrics;
    #[cfg(feature = "metrics")]
    pub mod metrics_service;
    #[cfg(feature = "metrics")]
    pub mod metrics_service_grpc;
    pub mod resource;
    #[cfg(feature = "traces")]
    pub mod trace;
    #[cfg(feature = "traces")]
    pub mod trace_config;
    #[cfg(feature = "traces")]
    pub mod trace_service;
    #[cfg(feature = "traces")]
    pub mod trace_service_grpc;
    #[cfg(feature = "zpages")]
    pub mod tracez;

    pub use crate::transform::common::grpcio::Attributes;
}
