#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExportLogsServiceRequest {
    /// An array of ResourceLogs.
    /// For data coming from a single resource this array will typically contain one
    /// element. Intermediary nodes (such as OpenTelemetry Collector) that receive
    /// data from multiple origins typically batch the data before forwarding further and
    /// in that case this array will contain multiple elements.
    #[prost(message, repeated, tag = "1")]
    pub resource_logs: ::prost::alloc::vec::Vec<
        super::super::super::logs::v1::ResourceLogs,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExportLogsServiceResponse {
    /// The details of a partially successful export request.
    ///
    /// If the request is only partially accepted
    /// (i.e. when the server accepts only parts of the data and rejects the rest)
    /// the server MUST initialize the `partial_success` field and MUST
    /// set the `rejected_<signal>` with the number of items it rejected.
    ///
    /// Servers MAY also make use of the `partial_success` field to convey
    /// warnings/suggestions to senders even when the request was fully accepted.
    /// In such cases, the `rejected_<signal>` MUST have a value of `0` and
    /// the `error_message` MUST be non-empty.
    ///
    /// A `partial_success` message with an empty value (rejected_<signal> = 0 and
    /// `error_message` = "") is equivalent to it not being set/present. Senders
    /// SHOULD interpret it the same way as in the full success case.
    #[prost(message, optional, tag = "1")]
    pub partial_success: ::core::option::Option<ExportLogsPartialSuccess>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExportLogsPartialSuccess {
    /// The number of rejected log records.
    ///
    /// A `rejected_<signal>` field holding a `0` value indicates that the
    /// request was fully accepted.
    #[prost(int64, tag = "1")]
    pub rejected_log_records: i64,
    /// A developer-facing human-readable message in English. It should be used
    /// either to explain why the server rejected parts of the data during a partial
    /// success or to convey warnings/suggestions during a full success. The message
    /// should offer guidance on how users can address such issues.
    ///
    /// error_message is an optional field. An error_message with an empty value
    /// is equivalent to it not being set.
    #[prost(string, tag = "2")]
    pub error_message: ::prost::alloc::string::String,
}
const METHOD_LOGS_SERVICE_EXPORT: ::grpcio::Method<
    ExportLogsServiceRequest,
    ExportLogsServiceResponse,
> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/opentelemetry.proto.collector.logs.v1.LogsService/Export",
    req_mar: ::grpcio::Marshaller {
        ser: ::grpcio::pr_ser,
        de: ::grpcio::pr_de,
    },
    resp_mar: ::grpcio::Marshaller {
        ser: ::grpcio::pr_ser,
        de: ::grpcio::pr_de,
    },
};
#[derive(Clone)]
pub struct LogsServiceClient {
    pub client: ::grpcio::Client,
}
impl LogsServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        LogsServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }
    pub fn export_opt(
        &self,
        req: &ExportLogsServiceRequest,
        opt: ::grpcio::CallOption,
    ) -> ::grpcio::Result<ExportLogsServiceResponse> {
        self.client.unary_call(&METHOD_LOGS_SERVICE_EXPORT, req, opt)
    }
    pub fn export(
        &self,
        req: &ExportLogsServiceRequest,
    ) -> ::grpcio::Result<ExportLogsServiceResponse> {
        self.export_opt(req, ::grpcio::CallOption::default())
    }
    pub fn export_async_opt(
        &self,
        req: &ExportLogsServiceRequest,
        opt: ::grpcio::CallOption,
    ) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<ExportLogsServiceResponse>> {
        self.client.unary_call_async(&METHOD_LOGS_SERVICE_EXPORT, req, opt)
    }
    pub fn export_async(
        &self,
        req: &ExportLogsServiceRequest,
    ) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<ExportLogsServiceResponse>> {
        self.export_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F)
    where
        F: ::std::future::Future<Output = ()> + Send + 'static,
    {
        self.client.spawn(f)
    }
}
pub trait LogsService {
    fn export(
        &mut self,
        ctx: ::grpcio::RpcContext,
        _req: ExportLogsServiceRequest,
        sink: ::grpcio::UnarySink<ExportLogsServiceResponse>,
    ) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}
pub fn create_logs_service<S: LogsService + Send + Clone + 'static>(
    s: S,
) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder
        .add_unary_handler(
            &METHOD_LOGS_SERVICE_EXPORT,
            move |ctx, req, resp| instance.export(ctx, req, resp),
        );
    builder.build()
}
