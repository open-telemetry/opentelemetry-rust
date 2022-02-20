// generated files from jaeger proto(https://github.com/jaegertracing/jaeger-idl/tree/main/proto/api_v2)
// DO NOT EDIT

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValue {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(enumeration = "ValueType", tag = "2")]
    pub v_type: i32,
    #[prost(string, tag = "3")]
    pub v_str: ::prost::alloc::string::String,
    #[prost(bool, tag = "4")]
    pub v_bool: bool,
    #[prost(int64, tag = "5")]
    pub v_int64: i64,
    #[prost(double, tag = "6")]
    pub v_float64: f64,
    #[prost(bytes = "vec", tag = "7")]
    pub v_binary: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Log {
    #[prost(message, optional, tag = "1")]
    pub timestamp: std::option::Option<::prost_types::Timestamp>,
    #[prost(message, repeated, tag = "2")]
    pub fields: std::vec::Vec<KeyValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpanRef {
    #[prost(bytes = "vec", tag = "1")]
    pub trace_id: std::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub span_id: std::vec::Vec<u8>,
    #[prost(enumeration = "SpanRefType", tag = "3")]
    pub ref_type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Process {
    #[prost(string, tag = "1")]
    pub service_name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub tags: std::vec::Vec<KeyValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Span {
    #[prost(bytes = "vec", tag = "1")]
    pub trace_id: std::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub span_id: std::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub operation_name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub references: std::vec::Vec<SpanRef>,
    #[prost(uint32, tag = "5")]
    pub flags: u32,
    #[prost(message, optional, tag = "6")]
    pub start_time: std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "7")]
    pub duration: std::option::Option<::prost_types::Duration>,
    #[prost(message, repeated, tag = "8")]
    pub tags: std::vec::Vec<KeyValue>,
    #[prost(message, repeated, tag = "9")]
    pub logs: std::vec::Vec<Log>,
    #[prost(message, optional, tag = "10")]
    pub process: std::option::Option<Process>,
    #[prost(string, tag = "11")]
    pub process_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "12")]
    pub warnings: std::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trace {
    #[prost(message, repeated, tag = "1")]
    pub spans: std::vec::Vec<Span>,
    #[prost(message, repeated, tag = "2")]
    pub process_map: std::vec::Vec<trace::ProcessMapping>,
    #[prost(string, repeated, tag = "3")]
    pub warnings: std::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `Trace`.
pub mod trace {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ProcessMapping {
        #[prost(string, tag = "1")]
        pub process_id: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub process: std::option::Option<super::Process>,
    }
}
/// Note that both Span and Batch may contain a Process.
/// This is different from the Thrift model which was only used
/// for transport, because Proto model is also used by the backend
/// as the domain model, where once a batch is received it is split
/// into individual spans which are all processed independently,
/// and therefore they all need a Process. As far as on-the-wire
/// semantics, both Batch and Spans in the same message may contain
/// their own instances of Process, with span.Process taking priority
/// over batch.Process.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Batch {
    #[prost(message, repeated, tag = "1")]
    pub spans: std::vec::Vec<Span>,
    #[prost(message, optional, tag = "2")]
    pub process: std::option::Option<Process>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DependencyLink {
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub child: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub call_count: u64,
    #[prost(string, tag = "4")]
    pub source: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ValueType {
    String = 0,
    Bool = 1,
    Int64 = 2,
    Float64 = 3,
    Binary = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpanRefType {
    ChildOf = 0,
    FollowsFrom = 1,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTraceRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub trace_id: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpansResponseChunk {
    #[prost(message, repeated, tag = "1")]
    pub spans: std::vec::Vec<Span>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArchiveTraceRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub trace_id: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArchiveTraceResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TraceQueryParameters {
    #[prost(string, tag = "1")]
    pub service_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub operation_name: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "3")]
    pub tags:
        std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub start_time_min: std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "5")]
    pub start_time_max: std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub duration_min: std::option::Option<::prost_types::Duration>,
    #[prost(message, optional, tag = "7")]
    pub duration_max: std::option::Option<::prost_types::Duration>,
    #[prost(int32, tag = "8")]
    pub search_depth: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindTracesRequest {
    #[prost(message, optional, tag = "1")]
    pub query: std::option::Option<TraceQueryParameters>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetServicesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetServicesResponse {
    #[prost(string, repeated, tag = "1")]
    pub services: std::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOperationsRequest {
    #[prost(string, tag = "1")]
    pub service: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub span_kind: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Operation {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub span_kind: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOperationsResponse {
    ///deprecated
    #[prost(string, repeated, tag = "1")]
    pub operation_names: std::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub operations: std::vec::Vec<Operation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDependenciesRequest {
    #[prost(message, optional, tag = "1")]
    pub start_time: std::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "2")]
    pub end_time: std::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDependenciesResponse {
    #[prost(message, repeated, tag = "1")]
    pub dependencies: std::vec::Vec<DependencyLink>,
}
#[doc = r" Generated client implementations."]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            QueryServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn get_trace(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTraceRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::SpansResponseChunk>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/GetTrace");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn archive_trace(
            &mut self,
            request: impl tonic::IntoRequest<super::ArchiveTraceRequest>,
        ) -> Result<tonic::Response<super::ArchiveTraceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/ArchiveTrace");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_traces(
            &mut self,
            request: impl tonic::IntoRequest<super::FindTracesRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::SpansResponseChunk>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/FindTraces");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn get_services(
            &mut self,
            request: impl tonic::IntoRequest<super::GetServicesRequest>,
        ) -> Result<tonic::Response<super::GetServicesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/GetServices");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_operations(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOperationsRequest>,
        ) -> Result<tonic::Response<super::GetOperationsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/GetOperations");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_dependencies(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDependenciesRequest>,
        ) -> Result<tonic::Response<super::GetDependenciesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/jaeger.api_v2.QueryService/GetDependencies");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
