#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TracezCounts {
    #[prost(string, tag = "1")]
    pub spanname: ::prost::alloc::string::String,
    /// \[A\]
    #[prost(uint32, repeated, tag = "2")]
    pub latency: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, tag = "3")]
    pub running: u32,
    #[prost(uint32, tag = "4")]
    pub error: u32,
}
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LatencyData {
    #[prost(bytes = "vec", tag = "1")]
    pub traceid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub spanid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub parentid: ::prost::alloc::vec::Vec<u8>,
    #[prost(fixed64, tag = "4")]
    pub starttime: u64,
    #[prost(fixed64, tag = "5")]
    pub endtime: u64,
    /// \[1\]
    #[prost(message, repeated, tag = "6")]
    pub attributes: ::prost::alloc::vec::Vec<super::super::common::v1::KeyValue>,
    /// \[2\]
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Event>,
    /// \[3\]
    #[prost(message, repeated, tag = "8")]
    pub links: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Link>,
}
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunningData {
    #[prost(bytes = "vec", tag = "1")]
    pub traceid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub spanid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub parentid: ::prost::alloc::vec::Vec<u8>,
    #[prost(fixed64, tag = "4")]
    pub starttime: u64,
    /// \[1\]
    #[prost(message, repeated, tag = "5")]
    pub attributes: ::prost::alloc::vec::Vec<super::super::common::v1::KeyValue>,
    /// \[2\]
    #[prost(message, repeated, tag = "6")]
    pub events: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Event>,
    /// \[3\]
    #[prost(message, repeated, tag = "7")]
    pub links: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Link>,
}
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ErrorData {
    #[prost(bytes = "vec", tag = "1")]
    pub traceid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub spanid: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub parentid: ::prost::alloc::vec::Vec<u8>,
    #[prost(fixed64, tag = "4")]
    pub starttime: u64,
    /// \[1\]
    #[prost(message, repeated, tag = "5")]
    pub attributes: ::prost::alloc::vec::Vec<super::super::common::v1::KeyValue>,
    /// \[2\]
    #[prost(message, repeated, tag = "6")]
    pub events: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Event>,
    /// \[3\]
    #[prost(message, repeated, tag = "7")]
    pub links: ::prost::alloc::vec::Vec<super::super::trace::v1::span::Link>,
    /// \[4\]
    #[prost(message, optional, tag = "8")]
    pub status: ::core::option::Option<super::super::trace::v1::Status>,
}
