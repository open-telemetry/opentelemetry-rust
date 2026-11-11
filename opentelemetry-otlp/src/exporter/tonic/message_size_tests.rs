use crate::{RetryPolicy, WithExportConfig, WithTonicConfig};
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
use opentelemetry::InstrumentationScope;
use opentelemetry_proto::tonic::collector::{
    logs::v1 as logs, metrics::v1 as metrics, trace::v1 as trace,
};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter, SdkLoggerProvider};
use opentelemetry_sdk::metrics::{data::ResourceMetrics, exporter::PushMetricExporter};
use opentelemetry_sdk::testing::trace::new_test_export_span_data;
use opentelemetry_sdk::trace::SpanExporter;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};

#[derive(Clone)]
struct Collector {
    requests: Arc<AtomicUsize>,
    response_size: usize,
}

macro_rules! impl_service {
    ($service:path, $request:ty, $response:ident, $partial:ident, $module:ident) => {
        #[tonic::async_trait]
        impl $service for Collector {
            async fn export(
                &self,
                _: tonic::Request<$request>,
            ) -> Result<tonic::Response<$module::$response>, tonic::Status> {
                self.requests.fetch_add(1, Ordering::SeqCst);
                Ok(tonic::Response::new($module::$response {
                    partial_success: Some($module::$partial {
                        error_message: "x".repeat(self.response_size),
                        ..Default::default()
                    }),
                }))
            }
        }
    };
}

impl_service!(
    trace::trace_service_server::TraceService,
    trace::ExportTraceServiceRequest,
    ExportTraceServiceResponse,
    ExportTracePartialSuccess,
    trace
);
impl_service!(
    metrics::metrics_service_server::MetricsService,
    metrics::ExportMetricsServiceRequest,
    ExportMetricsServiceResponse,
    ExportMetricsPartialSuccess,
    metrics
);
impl_service!(
    logs::logs_service_server::LogsService,
    logs::ExportLogsServiceRequest,
    ExportLogsServiceResponse,
    ExportLogsPartialSuccess,
    logs
);

struct TestServer {
    endpoint: String,
    requests: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn new(response_size: usize) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(AtomicUsize::new(0));
        let collector = Collector {
            requests: requests.clone(),
            response_size,
        };
        let task = tokio::spawn(async move {
            Server::builder()
                .add_service(trace::trace_service_server::TraceServiceServer::new(
                    collector.clone(),
                ))
                .add_service(metrics::metrics_service_server::MetricsServiceServer::new(
                    collector.clone(),
                ))
                .add_service(logs::logs_service_server::LogsServiceServer::new(collector))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        Self {
            endpoint,
            requests,
            task,
        }
    }

    fn configure<B: WithExportConfig + WithTonicConfig>(
        &self,
        builder: B,
        custom_channel: bool,
        encoding_limit: Option<usize>,
        decoding_limit: Option<usize>,
    ) -> B {
        let mut builder = builder
            .with_timeout(Duration::from_secs(5))
            .with_retry_policy(RetryPolicy::disabled());
        builder = if custom_channel {
            builder.with_channel(
                Channel::from_shared(self.endpoint.clone())
                    .unwrap()
                    .timeout(Duration::from_secs(5))
                    .connect_lazy(),
            )
        } else {
            builder.with_endpoint(self.endpoint.clone())
        };
        if let Some(limit) = encoding_limit {
            builder = builder.with_max_encoding_message_size(limit);
        }
        if let Some(limit) = decoding_limit {
            builder = builder.with_max_decoding_message_size(limit);
        }
        builder
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Clone, Copy, Debug)]
enum Signal {
    Traces,
    Metrics,
    Logs,
}

impl Signal {
    async fn export(
        self,
        server: &TestServer,
        custom_channel: bool,
        encoding_limit: Option<usize>,
        decoding_limit: Option<usize>,
    ) -> OTelSdkResult {
        match self {
            Self::Traces => {
                let exporter = server
                    .configure(
                        crate::SpanExporter::builder().with_tonic(),
                        custom_channel,
                        encoding_limit,
                        decoding_limit,
                    )
                    .build()
                    .unwrap();
                exporter.export(vec![new_test_export_span_data()]).await
            }
            Self::Metrics => {
                let exporter = server
                    .configure(
                        crate::MetricExporter::builder().with_tonic(),
                        custom_channel,
                        encoding_limit,
                        decoding_limit,
                    )
                    .build()
                    .unwrap();
                exporter.export(&ResourceMetrics::default()).await
            }
            Self::Logs => {
                let exporter = server
                    .configure(
                        crate::LogExporter::builder().with_tonic(),
                        custom_channel,
                        encoding_limit,
                        decoding_limit,
                    )
                    .build()
                    .unwrap();
                let provider = SdkLoggerProvider::builder().build();
                let mut record = provider.logger("message-size-test").create_log_record();
                record.set_observed_timestamp(SystemTime::now());
                let scope = InstrumentationScope::builder("message-size-test").build();
                exporter.export(LogBatch::new(&[(&record, &scope)])).await
            }
        }
    }
}

#[tokio::test]
async fn encoding_limits_apply_to_all_signals_and_custom_channels() {
    let server = TestServer::new(32).await;
    for signal in [Signal::Traces, Signal::Metrics, Signal::Logs] {
        for custom_channel in [false, true] {
            for limit in [None, Some(0), Some(1024 * 1024)] {
                let before = server.requests.load(Ordering::SeqCst);
                let result = signal.export(&server, custom_channel, limit, None).await;
                assert_eq!(
                    result.is_ok(),
                    limit != Some(0),
                    "{signal:?}, custom channel: {custom_channel}, limit: {limit:?}: {result:?}"
                );
                // An encoding rejection must happen before the collector receives the batch.
                assert_eq!(
                    server.requests.load(Ordering::SeqCst) - before,
                    usize::from(limit != Some(0))
                );
            }
        }
    }
}

#[tokio::test]
async fn decoding_limits_apply_to_all_signals_and_custom_channels() {
    for response_size in [32, 4 * 1024 * 1024 + 1] {
        let server = TestServer::new(response_size).await;
        for signal in [Signal::Traces, Signal::Metrics, Signal::Logs] {
            for custom_channel in [false, true] {
                for limit in [None, Some(0), Some(8 * 1024 * 1024)] {
                    let before = server.requests.load(Ordering::SeqCst);
                    let result = signal.export(&server, custom_channel, None, limit).await;
                    let expected_success = response_size < limit.unwrap_or(4 * 1024 * 1024);
                    assert_eq!(result.is_ok(), expected_success,
                        "{signal:?}, custom channel: {custom_channel}, response size: {response_size}, limit: {limit:?}: {result:?}");
                    // A decoding rejection happens after the request reaches the collector.
                    assert_eq!(server.requests.load(Ordering::SeqCst) - before, 1);
                }
            }
        }
    }
}
