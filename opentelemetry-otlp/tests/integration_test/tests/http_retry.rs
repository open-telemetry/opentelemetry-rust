#![cfg(all(unix, feature = "reqwest-client"))]

use std::time::Duration;

use anyhow::Result;
use integration_test_runner::fake_otlp_http::{
    CapturedHttpRequest, FakeOtlpHttpEndpoint, ScriptedHttpResponse,
};
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_otlp::{
    LogExporter, MetricExporter, RetryPolicy, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader;
use opentelemetry_sdk::metrics::{SdkMeterProvider, Temporality};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::SdkTracerProvider;
use prost::Message;

#[derive(Clone, Copy, Debug)]
enum Signal {
    Traces,
    Logs,
    Metrics,
}

impl Signal {
    const ALL: [Self; 3] = [Self::Traces, Self::Logs, Self::Metrics];

    fn path(self) -> &'static str {
        match self {
            Self::Traces => "/v1/traces",
            Self::Logs => "/v1/logs",
            Self::Metrics => "/v1/metrics",
        }
    }

    fn item_count(self, body: &[u8]) -> usize {
        match self {
            Self::Traces => {
                use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;

                ExportTraceServiceRequest::decode(body)
                    .expect("failed to decode OTLP trace request")
                    .resource_spans
                    .iter()
                    .flat_map(|resource| &resource.scope_spans)
                    .map(|scope| scope.spans.len())
                    .sum()
            }
            Self::Logs => {
                use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;

                ExportLogsServiceRequest::decode(body)
                    .expect("failed to decode OTLP log request")
                    .resource_logs
                    .iter()
                    .flat_map(|resource| &resource.scope_logs)
                    .map(|scope| scope.log_records.len())
                    .sum()
            }
            Self::Metrics => {
                use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

                ExportMetricsServiceRequest::decode(body)
                    .expect("failed to decode OTLP metric request")
                    .resource_metrics
                    .iter()
                    .flat_map(|resource| &resource.scope_metrics)
                    .map(|scope| scope.metrics.len())
                    .sum()
            }
        }
    }
}

fn retry_policy() -> RetryPolicy {
    RetryPolicy::default()
        .with_max_retries(3)
        .with_initial_delay(Duration::from_millis(10))
        .with_max_delay(Duration::from_millis(10))
        .with_max_jitter(Duration::ZERO)
}

fn assert_otlp_requests(signal: Signal, requests: &[CapturedHttpRequest], expected_count: usize) {
    assert_eq!(requests.len(), expected_count);
    for request in requests {
        assert_eq!(
            request.request_line,
            format!("POST {} HTTP/1.1", signal.path())
        );
        assert_eq!(
            request.headers.get("content-type").map(String::as_str),
            Some("application/x-protobuf")
        );
        assert!(!request.body.is_empty(), "OTLP request body was empty");
        assert_eq!(signal.item_count(&request.body), 1);
    }
}

async fn export_to(
    signal: Signal,
    endpoint: &FakeOtlpHttpEndpoint,
) -> opentelemetry_sdk::error::OTelSdkResult {
    let endpoint = endpoint.endpoint(signal.path());
    match signal {
        Signal::Traces => {
            let exporter = SpanExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .with_timeout(Duration::from_secs(5))
                .with_retry_policy(retry_policy())
                .build()
                .expect("failed to build OTLP HTTP span exporter");

            let processor = BatchSpanProcessor::builder(exporter, Tokio).build();
            let provider = SdkTracerProvider::builder()
                .with_span_processor(processor)
                .build();
            tokio::task::spawn_blocking(move || {
                let tracer = provider.tracer("http-retry-test");
                let mut span = tracer.start("http-retry-test");
                span.end();
                provider.shutdown()
            })
            .await
            .expect("OTLP trace pipeline task panicked")
        }
        Signal::Logs => {
            let exporter = LogExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .with_timeout(Duration::from_secs(5))
                .with_retry_policy(retry_policy())
                .build()
                .expect("failed to build OTLP HTTP log exporter");

            let processor = BatchLogProcessor::builder(exporter, Tokio).build();
            let provider = SdkLoggerProvider::builder()
                .with_log_processor(processor)
                .build();
            tokio::task::spawn_blocking(move || {
                let logger = provider.logger("http-retry-test");
                let mut record = logger.create_log_record();
                record.set_observed_timestamp(std::time::SystemTime::now());
                record.set_body("http-retry-test".into());
                logger.emit(record);
                provider.shutdown()
            })
            .await
            .expect("OTLP log pipeline task panicked")
        }
        Signal::Metrics => {
            let exporter = MetricExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .with_timeout(Duration::from_secs(5))
                .with_retry_policy(retry_policy())
                .with_temporality(Temporality::Cumulative)
                .build()
                .expect("failed to build OTLP HTTP metric exporter");

            let reader = PeriodicReader::builder(exporter, Tokio).build();
            let provider = SdkMeterProvider::builder().with_reader(reader).build();
            tokio::task::spawn_blocking(move || {
                let meter = provider.meter("http-retry-test");
                let counter = meter.u64_counter("http-retry-test").build();
                counter.add(1, &[]);
                provider.shutdown()
            })
            .await
            .expect("OTLP metric pipeline task panicked")
        }
    }
}

async fn assert_retries_service_unavailable(signal: Signal) -> Result<()> {
    let endpoint = FakeOtlpHttpEndpoint::start(vec![
        ScriptedHttpResponse::new(503),
        ScriptedHttpResponse::new(200),
    ])
    .await?;

    export_to(signal, &endpoint).await?;

    assert_otlp_requests(signal, &endpoint.requests(), 2);
    Ok(())
}

async fn assert_does_not_retry_bad_request(signal: Signal) -> Result<()> {
    let endpoint = FakeOtlpHttpEndpoint::start(vec![ScriptedHttpResponse::new(400)]).await?;

    assert!(export_to(signal, &endpoint).await.is_err());

    assert_otlp_requests(signal, &endpoint.requests(), 1);
    Ok(())
}

async fn assert_honors_retry_after(signal: Signal) -> Result<()> {
    let endpoint = FakeOtlpHttpEndpoint::start(vec![
        ScriptedHttpResponse::new(429).with_header("Retry-After", "1"),
        ScriptedHttpResponse::new(200),
    ])
    .await?;

    export_to(signal, &endpoint).await?;

    let requests = endpoint.requests();
    assert_otlp_requests(signal, &requests, 2);
    assert!(
        requests[1]
            .received_at
            .duration_since(requests[0].received_at)
            >= Duration::from_secs(1),
        "{signal:?} exporter retried before the Retry-After delay elapsed"
    );
    Ok(())
}

#[tokio::test]
async fn retries_service_unavailable_then_succeeds() -> Result<()> {
    let [traces, logs, metrics] = Signal::ALL;
    tokio::try_join!(
        assert_retries_service_unavailable(traces),
        assert_retries_service_unavailable(logs),
        assert_retries_service_unavailable(metrics)
    )?;
    Ok(())
}

#[tokio::test]
async fn does_not_retry_bad_request() -> Result<()> {
    let [traces, logs, metrics] = Signal::ALL;
    tokio::try_join!(
        assert_does_not_retry_bad_request(traces),
        assert_does_not_retry_bad_request(logs),
        assert_does_not_retry_bad_request(metrics)
    )?;
    Ok(())
}

#[tokio::test]
async fn honors_retry_after_before_retrying() -> Result<()> {
    let [traces, logs, metrics] = Signal::ALL;
    tokio::try_join!(
        assert_honors_retry_after(traces),
        assert_honors_retry_after(logs),
        assert_honors_retry_after(metrics)
    )?;
    Ok(())
}
