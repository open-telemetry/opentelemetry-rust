//! OTEL metric exporter
//!
//! Defines a [MetricExporter] to send metric data to backend via OTLP protocol.
//!

#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
use crate::HasExportConfig;

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::{exporter::http::HttpExporterBuilder, HasHttpConfig, HttpExporterBuilderSet};

#[cfg(feature = "grpc-tonic")]
use crate::{exporter::tonic::TonicExporterBuilder, HasTonicConfig, TonicExporterBuilderSet};

use crate::{ExporterBuildError, NoExporterBuilderSet};

use core::fmt;
use opentelemetry_sdk::error::OTelSdkResult;

use opentelemetry_sdk::metrics::{
    data::ResourceMetrics, exporter::PushMetricExporter, Temporality,
};
use std::fmt::{Debug, Formatter};
use std::time::Duration;

/// Target to which the exporter is going to send metrics, defaults to https://localhost:4317/v1/metrics.
/// Learn about the relationship between this constant and default/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Max waiting time for the backend to process each metrics batch, defaults to 10s.
pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";
/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_METRICS_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_METRICS_COMPRESSION";
/// Key-value pairs to be used as headers associated with gRPC or HTTP requests
/// for sending metrics.
/// Example: `k1=v1,k2=v2`
/// Note: this is only supported for HTTP.
pub const OTEL_EXPORTER_OTLP_METRICS_HEADERS: &str = "OTEL_EXPORTER_OTLP_METRICS_HEADERS";
/// Temporality preference for metrics, defaults to cumulative.
/// Accepted values: `cumulative`, `delta`, `lowmemory` (case-insensitive).
pub const OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: &str =
    "OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE";

/// A builder for creating a new [MetricExporter].
#[derive(Debug, Default, Clone)]
pub struct MetricExporterBuilder<C> {
    client: C,
    temporality: Option<Temporality>,
}

impl MetricExporterBuilder<NoExporterBuilderSet> {
    /// Create a new [MetricExporterBuilder] with default settings.
    pub fn new() -> Self {
        MetricExporterBuilder::default()
    }
}

impl<C> MetricExporterBuilder<C> {
    /// With the gRPC Tonic transport.
    #[cfg(feature = "grpc-tonic")]
    pub fn with_tonic(self) -> MetricExporterBuilder<TonicExporterBuilderSet> {
        MetricExporterBuilder {
            client: TonicExporterBuilderSet(TonicExporterBuilder::default()),
            temporality: self.temporality,
        }
    }

    /// With the HTTP transport.
    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    pub fn with_http(self) -> MetricExporterBuilder<HttpExporterBuilderSet> {
        MetricExporterBuilder {
            client: HttpExporterBuilderSet(HttpExporterBuilder::default()),
            temporality: self.temporality,
        }
    }

    /// Set the temporality for the metrics.
    ///
    /// Note: Programmatically setting this will override any value set via the
    /// `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` environment variable.
    pub fn with_temporality(self, temporality: Temporality) -> MetricExporterBuilder<C> {
        MetricExporterBuilder {
            client: self.client,
            temporality: Some(temporality),
        }
    }
}

/// Resolve the temporality from the provided config or environment variable.
///
/// Priority: programmatic config > env var > default (`Cumulative`).
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
fn resolve_temporality(provided: Option<Temporality>) -> Result<Temporality, ExporterBuildError> {
    if let Some(temporality) = provided {
        return Ok(temporality);
    }
    if let Ok(val) = std::env::var(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE) {
        return val.parse::<Temporality>().map_err(|_| {
            ExporterBuildError::InternalFailure(format!(
                "Invalid value for {OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE}: {val}. Expected: cumulative, delta, or lowmemory"
            ))
        });
    }
    Ok(Temporality::default())
}

#[cfg(feature = "grpc-tonic")]
impl MetricExporterBuilder<TonicExporterBuilderSet> {
    /// Build the [MetricExporter] with the gRPC Tonic transport.
    pub fn build(self) -> Result<MetricExporter, ExporterBuildError> {
        let temporality = resolve_temporality(self.temporality)?;
        let exporter = self.client.0.build_metrics_exporter(temporality)?;
        opentelemetry::otel_debug!(name: "MetricExporterBuilt");
        Ok(exporter)
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl MetricExporterBuilder<HttpExporterBuilderSet> {
    /// Build the [MetricExporter] with the HTTP transport.
    pub fn build(self) -> Result<MetricExporter, ExporterBuildError> {
        let temporality = resolve_temporality(self.temporality)?;
        let exporter = self.client.0.build_metrics_exporter(temporality)?;
        Ok(exporter)
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasExportConfig for MetricExporterBuilder<TonicExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasExportConfig for MetricExporterBuilder<HttpExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasTonicConfig for MetricExporterBuilder<TonicExporterBuilderSet> {
    fn tonic_config(&mut self) -> &mut crate::TonicConfig {
        &mut self.client.0.tonic_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasHttpConfig for MetricExporterBuilder<HttpExporterBuilderSet> {
    fn http_client_config(&mut self) -> &mut crate::exporter::http::HttpConfig {
        &mut self.client.0.http_config
    }
}

/// An interface for OTLP metrics clients
pub(crate) trait MetricsClient: fmt::Debug + Send + Sync + 'static {
    fn export(
        &self,
        metrics: &ResourceMetrics,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send;
    fn shutdown(&self) -> OTelSdkResult;
}

/// Export metrics in OTEL format.
pub struct MetricExporter {
    client: SupportedTransportClient,
    temporality: Temporality,
}

#[derive(Debug)]
enum SupportedTransportClient {
    #[cfg(feature = "grpc-tonic")]
    Tonic(crate::exporter::tonic::metrics::TonicMetricsClient),
    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    Http(crate::exporter::http::OtlpHttpClient),
}

impl Debug for MetricExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricExporter").finish()
    }
}

impl PushMetricExporter for MetricExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        match &self.client {
            #[cfg(feature = "grpc-tonic")]
            SupportedTransportClient::Tonic(client) => client.export(metrics).await,
            #[cfg(any(feature = "http-proto", feature = "http-json"))]
            SupportedTransportClient::Http(client) => client.export(metrics).await,
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        // this component is stateless
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult {
        match &self.client {
            #[cfg(feature = "grpc-tonic")]
            SupportedTransportClient::Tonic(client) => client.shutdown(),
            #[cfg(any(feature = "http-proto", feature = "http-json"))]
            SupportedTransportClient::Http(client) => client.shutdown(),
        }
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

impl MetricExporter {
    /// Obtain a builder to configure a [MetricExporter].
    pub fn builder() -> MetricExporterBuilder<NoExporterBuilderSet> {
        MetricExporterBuilder::default()
    }

    #[cfg(feature = "grpc-tonic")]
    pub(crate) fn from_tonic(
        client: crate::exporter::tonic::metrics::TonicMetricsClient,
        temporality: Temporality,
    ) -> Self {
        Self {
            client: SupportedTransportClient::Tonic(client),
            temporality,
        }
    }

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    pub(crate) fn from_http(
        client: crate::exporter::http::OtlpHttpClient,
        temporality: Temporality,
    ) -> Self {
        Self {
            client: SupportedTransportClient::Http(client),
            temporality,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::tests::run_env_test;

    #[test]
    fn code_config_overrides_env_var() {
        run_env_test(
            vec![(
                OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE,
                "cumulative",
            )],
            || {
                let result = resolve_temporality(Some(Temporality::Delta)).unwrap();
                assert_eq!(result, Temporality::Delta);
            },
        );
    }

    #[test]
    fn env_var_sets_delta() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE, "delta")],
            || {
                let result = resolve_temporality(None).unwrap();
                assert_eq!(result, Temporality::Delta);
            },
        );
    }

    #[test]
    fn env_var_sets_lowmemory() {
        run_env_test(
            vec![(
                OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE,
                "lowmemory",
            )],
            || {
                let result = resolve_temporality(None).unwrap();
                assert_eq!(result, Temporality::LowMemory);
            },
        );
    }

    #[test]
    fn env_var_case_insensitive() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE, "Delta")],
            || {
                let result = resolve_temporality(None).unwrap();
                assert_eq!(result, Temporality::Delta);
            },
        );
    }

    #[test]
    fn invalid_env_var_returns_error() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE, "invalid")],
            || {
                let result = resolve_temporality(None);
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_use_default_when_nothing_set() {
        temp_env::with_var_unset(OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE, || {
            let result = resolve_temporality(None).unwrap();
            assert_eq!(result, Temporality::Cumulative);
        });
    }
}
