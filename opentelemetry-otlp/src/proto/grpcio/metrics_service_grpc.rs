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

const METHOD_METRICS_SERVICE_EXPORT: ::grpcio::Method<super::metrics_service::ExportMetricsServiceRequest, super::metrics_service::ExportMetricsServiceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct MetricsServiceClient {
    client: ::grpcio::Client,
}

impl MetricsServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        MetricsServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn export_opt(&self, req: &super::metrics_service::ExportMetricsServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::metrics_service::ExportMetricsServiceResponse> {
        self.client.unary_call(&METHOD_METRICS_SERVICE_EXPORT, req, opt)
    }

    pub fn export(&self, req: &super::metrics_service::ExportMetricsServiceRequest) -> ::grpcio::Result<super::metrics_service::ExportMetricsServiceResponse> {
        self.export_opt(req, ::grpcio::CallOption::default())
    }

    pub fn export_async_opt(&self, req: &super::metrics_service::ExportMetricsServiceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::metrics_service::ExportMetricsServiceResponse>> {
        self.client.unary_call_async(&METHOD_METRICS_SERVICE_EXPORT, req, opt)
    }

    pub fn export_async(&self, req: &super::metrics_service::ExportMetricsServiceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::metrics_service::ExportMetricsServiceResponse>> {
        self.export_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures_core::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait MetricsService {
    fn export(&mut self, ctx: ::grpcio::RpcContext, req: super::metrics_service::ExportMetricsServiceRequest, sink: ::grpcio::UnarySink<super::metrics_service::ExportMetricsServiceResponse>);
}

pub fn create_metrics_service<S: MetricsService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_METRICS_SERVICE_EXPORT, move |ctx, req, resp| {
        instance.export(ctx, req, resp)
    });
    builder.build()
}
