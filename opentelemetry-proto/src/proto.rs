#[cfg(feature = "gen-tonic")]
pub mod tonic {
    pub mod collector {
        pub mod metrics {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.collector.metrics.v1.rs"));
            }
        }

        pub mod trace {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.collector.trace.v1.rs"));
            }
        }
    }

    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.common.v1.rs"));
        }
    }

    pub mod metrics {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.metrics.v1.rs"));
        }
    }

    pub mod resource {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.resource.v1.rs"));
        }
    }

    pub mod trace {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.trace.v1.rs"));
        }
    }

    pub use crate::transform::common::tonic::Attributes;

    #[cfg(feature = "metrics")]
    pub use crate::transform::metrics::tonic::FromNumber;
}

#[cfg(feature = "gen-protoc")]
pub mod grpcio {
    pub mod common;
    pub mod metrics;
    pub mod metrics_service;
    pub mod metrics_service_grpc;
    pub mod resource;
    pub mod trace;
    pub mod trace_config;
    pub mod trace_service;
    pub mod trace_service_grpc;
    pub mod tracez;

    pub use crate::transform::common::grpcio::Attributes;
}
