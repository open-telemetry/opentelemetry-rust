#[cfg(feature = "gen-tonic-messages")]
#[path = "proto/tonic"]
/// Generated files using [`tonic`](https://docs.rs/crate/tonic) and [`prost`](https://docs.rs/crate/prost)
pub mod tonic {
    #[cfg(feature = "with-serde")]
    #[path = "../serializers.rs"]
    pub(crate) mod serializers;

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

        #[cfg(feature = "trace")]
        #[path = ""]
        pub mod trace {
            #[path = "opentelemetry.proto.collector.trace.v1.rs"]
            pub mod v1;
        }

        #[cfg(feature = "profiles")]
        #[path = ""]
        pub mod profiles {
            #[path = "opentelemetry.proto.collector.profiles.v1development.rs"]
            pub mod v1development;
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
    #[cfg(feature = "trace")]
    #[path = ""]
    pub mod trace {
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in zpages.
    #[cfg(feature = "zpages")]
    #[path = ""]
    pub mod tracez {
        #[path = "opentelemetry.proto.tracez.v1.rs"]
        pub mod v1;
    }

    /// Generated types used in zpages.
    #[cfg(feature = "profiles")]
    #[path = ""]
    pub mod profiles {
        #[path = "opentelemetry.proto.profiles.v1development.rs"]
        pub mod v1development;
    }

    pub use crate::transform::common::tonic::Attributes;
}

#[cfg(feature = "gen-json")]
#[path = "proto/json"]
/// Generated files for OTLP/JSON: the same messages as [`tonic`](self::tonic) with
/// [`serde`](https://docs.rs/crate/serde) support and no [`prost`](https://docs.rs/crate/prost) dependency.
pub mod json {
    #[path = "../serializers.rs"]
    pub(crate) mod serializers;

    /// Request and response messages
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

        #[cfg(feature = "trace")]
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
    #[cfg(feature = "trace")]
    #[path = ""]
    pub mod trace {
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }
}
