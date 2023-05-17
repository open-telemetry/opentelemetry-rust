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

const METHOD_LOGS_SERVICE_EXPORT: ::grpcio::Method<super::logs_service::ExportLogsServiceRequest, super::logs_service::ExportLogsServiceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/opentelemetry.proto.collector.logs.v1.LogsService/Export",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct LogsServiceClient {
    client: ::grpcio::Client,
}

impl LogsServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        LogsServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn export_opt(&self, req: &super::logs_service::ExportLogsServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::logs_service::ExportLogsServiceResponse> {
        self.client.unary_call(&METHOD_LOGS_SERVICE_EXPORT, req, opt)
    }

    pub fn export(&self, req: &super::logs_service::ExportLogsServiceRequest) -> ::grpcio::Result<super::logs_service::ExportLogsServiceResponse> {
        self.export_opt(req, ::grpcio::CallOption::default())
    }

    pub fn export_async_opt(&self, req: &super::logs_service::ExportLogsServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::logs_service::ExportLogsServiceResponse>> {
        self.client.unary_call_async(&METHOD_LOGS_SERVICE_EXPORT, req, opt)
    }

    pub fn export_async(&self, req: &super::logs_service::ExportLogsServiceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::logs_service::ExportLogsServiceResponse>> {
        self.export_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait LogsService {
    fn export(&mut self, ctx: ::grpcio::RpcContext, req: super::logs_service::ExportLogsServiceRequest, sink: ::grpcio::UnarySink<super::logs_service::ExportLogsServiceResponse>);
}

pub fn create_logs_service<S: LogsService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_LOGS_SERVICE_EXPORT, move |ctx, req, resp| {
        instance.export(ctx, req, resp)
    });
    builder.build()
}
