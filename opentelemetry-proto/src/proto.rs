#[cfg(feature = "gen-tonic")]
pub mod tonic {
    pub mod collector {
        pub mod logs {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.logs.v1");
            }
        }

        pub mod metrics {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.metrics.v1");
            }
        }

        pub mod trace {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.trace.v1");
            }
        }
    }

    pub mod common {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.common.v1");
        }
    }

    pub mod logs {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.logs.v1");
        }
    }

    pub mod metrics {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.metrics.v1");
        }
    }

    pub mod resource {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.resource.v1");
        }
    }

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
