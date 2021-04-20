#[cfg(feature = "tonic")]
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

#[cfg(feature = "tonic")]
pub mod common {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.common.v1.rs"));
    }
}

#[cfg(feature = "tonic")]
pub mod metrics {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.metrics.v1.rs"));
    }
}

#[cfg(feature = "tonic")]
pub mod resource {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.resource.v1.rs"));
    }
}

#[cfg(feature = "tonic")]
pub mod trace {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/tonic", "/opentelemetry.proto.trace.v1.rs"));
    }
}

#[cfg(feature="http-proto")]
pub mod prost {
    pub mod collector {
        pub mod metrics {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.collector.metrics.v1.rs"));
            }
        }

        pub mod trace {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.collector.trace.v1.rs"));
            }
        }
    }

    pub mod common {
        pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.common.v1.rs"));
        }
    }

    pub mod metrics {
        pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.metrics.v1.rs"));
        }
    }

    pub mod resource {
        pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.resource.v1.rs"));
        }
    }

    pub mod trace {
        pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/prost", "/opentelemetry.proto.trace.v1.rs"));
        }
    }
}

#[cfg(feature = "grpc-sys")]
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
