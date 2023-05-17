#[cfg(feature = "gen-tonic")]
#[path = "proto/tonic"]
/// Generated files using [`tonic`](https://docs.rs/crate/grpcio) and [`prost`](https://docs.rs/crate/protobuf/latest)
pub mod tonic {
    /// Service stub and clients
    #[path = ""]
    pub mod collector {
        #[cfg(feature = "logs")]
        #[path = ""]
        pub mod logs {
            #[path = "opentelemetry.proto.collector.logs.v1.rs"]
            pub mod v1;
        }

        #[cfg(feature = "metrics")]
        #[path = ""]
        pub mod metrics {
            #[path = "opentelemetry.proto.collector.metrics.v1.rs"]
            pub mod v1;
        }

        #[cfg(feature = "traces")]
        #[path = ""]
        pub mod trace {
            #[path = "opentelemetry.proto.collector.trace.v1.rs"]
            pub mod v1;
        }
    }

    /// Common types used across all signals
    #[path = ""]
    pub mod common {
        #[path = "opentelemetry.proto.common.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in logging.
    #[cfg(feature = "logs")]
    #[path = ""]
    pub mod logs {
        #[path = "opentelemetry.proto.logs.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in metrics.
    #[cfg(feature = "metrics")]
    #[path = ""]
    pub mod metrics {
        #[path = "opentelemetry.proto.metrics.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in resources.
    #[path = ""]
    pub mod resource {
        #[path = "opentelemetry.proto.resource.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in traces.
    #[cfg(feature = "traces")]
    #[path = ""]
    pub mod trace {
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }

    pub use crate::transform::common::tonic::Attributes;
}

#[cfg(feature = "gen-protoc")]
/// Generated files using [`grpcio`](https://docs.rs/crate/grpcio) and [`protobuf`](https://docs.rs/crate/protobuf/latest)
pub mod grpcio {
    pub mod common;
    #[cfg(feature = "logs")]
    pub mod logs;
    #[cfg(feature = "logs")]
    pub mod logs_service;
    #[cfg(feature = "logs")]
    pub mod logs_service_grpc;
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
    pub mod trace_service;
    #[cfg(feature = "traces")]
    pub mod trace_service_grpc;
    #[cfg(feature = "zpages")]
    pub mod tracez;

    pub use crate::transform::common::grpcio::Attributes;
}
