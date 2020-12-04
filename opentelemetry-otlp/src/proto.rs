#[cfg(feature = "tonic")]
pub(crate) mod collector {
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

#[cfg(feature = "tonic")]
pub(crate) mod common {
    pub mod v1 {
        tonic::include_proto!("opentelemetry.proto.common.v1");
    }
}

#[cfg(feature = "tonic")]
pub(crate) mod metrics {
    pub mod v1 {
        tonic::include_proto!("opentelemetry.proto.metrics.v1");
    }
}

#[cfg(feature = "tonic")]
pub(crate) mod resource {
    pub mod v1 {
        tonic::include_proto!("opentelemetry.proto.resource.v1");
    }
}

#[cfg(feature = "tonic")]
pub(crate) mod trace {
    pub mod v1 {
        tonic::include_proto!("opentelemetry.proto.trace.v1");
    }
}

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
pub(crate) mod grpcio {
    pub(crate) mod common;
    pub(crate) mod metrics;
    pub(crate) mod metrics_service;
    pub(crate) mod metrics_service_grpc;
    pub(crate) mod resource;
    pub(crate) mod trace;
    pub(crate) mod trace_config;
    pub(crate) mod trace_service;
    pub(crate) mod trace_service_grpc;
}
