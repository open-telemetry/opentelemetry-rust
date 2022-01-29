// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_TRACE_SERVICE_EXPORT: ::grpcio::Method<super::trace_service::ExportTraceServiceRequest, super::trace_service::ExportTraceServiceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/opentelemetry.proto.collector.trace.v1.TraceService/Export",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct TraceServiceClient {
    client: ::grpcio::Client,
}

impl TraceServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        TraceServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn export_opt(&self, req: &super::trace_service::ExportTraceServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::trace_service::ExportTraceServiceResponse> {
        self.client.unary_call(&METHOD_TRACE_SERVICE_EXPORT, req, opt)
    }

    pub fn export(&self, req: &super::trace_service::ExportTraceServiceRequest) -> ::grpcio::Result<super::trace_service::ExportTraceServiceResponse> {
        self.export_opt(req, ::grpcio::CallOption::default())
    }

    pub fn export_async_opt(&self, req: &super::trace_service::ExportTraceServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::trace_service::ExportTraceServiceResponse>> {
        self.client.unary_call_async(&METHOD_TRACE_SERVICE_EXPORT, req, opt)
    }

    pub fn export_async(&self, req: &super::trace_service::ExportTraceServiceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::trace_service::ExportTraceServiceResponse>> {
        self.export_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait TraceService {
    fn export(&mut self, ctx: ::grpcio::RpcContext, req: super::trace_service::ExportTraceServiceRequest, sink: ::grpcio::UnarySink<super::trace_service::ExportTraceServiceResponse>);
}

pub fn create_trace_service<S: TraceService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_TRACE_SERVICE_EXPORT, move |ctx, req, resp| {
        instance.export(ctx, req, resp)
    });
    builder.build()
}
