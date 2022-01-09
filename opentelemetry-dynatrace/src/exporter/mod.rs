//! OpenTelemetry Dynatrace Exporter Configuration
//!
/// Configuration for the Dynatrace exporter.
///
/// ## Examples
///
/// ```no_run
/// use opentelemetry_dynatrace::ExportConfig;
/// # fn main() {
/// let exporter_config = ExportConfig::default()
///     .with_token("*****".to_string());
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ExportConfig {
    /// The address of the Dynatrace endpoint.
    ///
    /// # Examples
    ///
    /// * Managed https://{your-domain}/e/{your-environment-id}/api/v2/metrics/ingest
    /// * SaaS https://{your-environment-id}.live.dynatrace.com/api/v2/metrics/ingest
    /// * Environment ActiveGate https://{your-activegate-domain}/e/{your-environment-id}/api/v2/metrics/ingest
    ///
    /// If no endpoint is defined, the endpoint of the locally installed Dynatrace OneAgent will be used as a fallback.
    pub endpoint: Option<String>,

    /// The API token for authentication.
    ///
    /// Authentication is not required when using the locally installed Dynatrace OneAgent as an endpoint.
    pub token: Option<String>,
}

impl ExportConfig {
    /// Set the address of the Dynatrace endoint.
    pub fn with_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the API token for authentication.
    pub fn with_token<T: Into<String>>(mut self, token: T) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the export configuration. This will override all previous configuration.
    pub fn with_export_config(mut self, export_config: ExportConfig) -> Self {
        self.endpoint = export_config.endpoint;
        self.token = export_config.token;
        self
    }
}
