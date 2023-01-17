use std::sync::Arc;

use opentelemetry::sdk::{
    export::metrics::aggregation,
    metrics::{controllers, processors, selectors},
    trace as sdktrace, Resource,
};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, MeterProvider, MetricsError},
    trace::TraceError,
    Context, Key, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};

/* OTLP Tracer Handler */

/// Create Global Tracer
pub fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    // Setup
    let svc_name = "svc_warp_api"; // env::var("SVC_NAME").expect("Env 'SVC_NAME' not found");
    let otlp_host = "http://127.0.0.1:4317"; // env::var("OTLP_HOST").expect("Env 'OTLP_HOST' not found");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_host),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                svc_name.to_string(),
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}

/// Create Global Prometheus Metrics
pub fn init_prometheus_metrics(name: &'static str) -> PrometheusMetricsHandler {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([0.1, 0.5, 2.0, 5.0, 10.0, 50.0, 100.0, 500.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .with_resource(Resource::new(vec![KeyValue::new(
        "svc_customer_api",
        "metrics",
    )]))
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();
    let meter = exporter
        .meter_provider()
        .unwrap()
        .versioned_meter(name, None, None);

    PrometheusMetricsHandler::new(exporter, meter)
}

// HTTP Private Attributes
const METER_COUNTER_KEY: &str = "http.counter";
const METER_LATENCY_KEY: &str = "http.latency";

// HTTP Reusable Attributes
pub const SUCCESS: &str = "success";
pub const ERROR: &str = "error";
pub const STATUS_KEY: Key = Key::from_static_str("status");
pub const HTTP_METHOD: Key = Key::from_static_str("http.method");
pub const HTTP_ENDPOINT: Key = Key::from_static_str("http.endpoint");
pub const HTTP_STATUS_CODE: Key = Key::from_static_str("http.status_code");

/* HTTP Metrics Handler */

#[derive(Debug, Clone)]
pub struct HttpMetrics {
    http_counter: Counter<u64>,
    http_latency: Histogram<f64>,
}

impl HttpMetrics {
    pub fn new(meter: Meter) -> Self {
        let http_counter = meter
            .u64_counter(METER_COUNTER_KEY)
            .with_description("HTTP counter requests per endpoint")
            .init();

        let http_latency = meter
            .f64_histogram(METER_LATENCY_KEY)
            .with_description("HTTP request latency seconds per endpoint")
            .init();

        HttpMetrics {
            http_counter,
            http_latency,
        }
    }

    fn make_counter_attr(method: &str, endpoint: &str, status_code: &str) -> Vec<KeyValue> {
        vec![
            HTTP_METHOD.string(method.to_owned()),
            HTTP_ENDPOINT.string(endpoint.to_owned()),
            HTTP_STATUS_CODE.string(status_code.to_owned()),
        ]
    }

    fn make_latency_attr(method: &str, endpoint: &str) -> Vec<KeyValue> {
        vec![
            HTTP_METHOD.string(method.to_owned()),
            HTTP_ENDPOINT.string(endpoint.to_owned()),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct HttpMetricsBuilder {
    http_metrics: Arc<HttpMetrics>,
}

impl HttpMetricsBuilder {
    pub fn new(http_metrics: Arc<HttpMetrics>) -> Self {
        HttpMetricsBuilder { http_metrics }
    }

    pub fn http_request_counter(&self, cx: &Context, method: &str, path: &str, status_code: &str) {
        self.http_metrics.http_counter.add(
            cx,
            1,
            &HttpMetrics::make_counter_attr(method, path, status_code),
        );
    }

    pub fn http_request_latency(&self, cx: &Context, method: &str, path: &str, time: f64) {
        self.http_metrics.http_latency.record(
            cx,
            time,
            &HttpMetrics::make_latency_attr(method, path),
        );
    }
}

/// Prometheus request metrics service
#[derive(Clone, Debug)]
pub struct PrometheusMetricsHandler {
    exporter: PrometheusExporter,
    meter: Meter,
}

impl PrometheusMetricsHandler {
    /// Build a route to serve Prometheus metrics
    pub fn new(exporter: PrometheusExporter, meter: Meter) -> Self {
        Self { exporter, meter }
    }

    pub fn receive_meter(&self) -> Meter {
        self.meter.clone()
    }

    pub fn metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metrics_pool = self.exporter.registry().gather();
        let mut buf = Vec::new();
        if let Err(e) = encoder.encode(&metrics_pool[..], &mut buf) {
            global::handle_error(MetricsError::Other(e.to_string()));
        }

        String::from_utf8(buf).unwrap_or_default()
    }
}
