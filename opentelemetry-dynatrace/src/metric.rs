//! Dynatrace Metric Exporter.
//!
//! Defines an `Exporter` to send metric data to Dynatrace using the [Dynatrace Metrics ingestion protocol].
//!
//! [Metrics ingestion protocol]: https://www.dynatrace.com/support/help/how-to-use-dynatrace/metrics/metric-ingestion/metric-ingestion-protocol/
#![allow(unused_attributes)]
use crate::exporter::ExportConfig;
use crate::transform::record_to_metric_line;
use crate::transform::{DimensionSet, MetricLine};
use crate::{DynatraceExporterBuilder, DynatracePipelineBuilder, Error};
use http::{
    header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Method, Uri, Version,
};
use opentelemetry::metrics::Result;
use opentelemetry::{global, Context};
use opentelemetry_http::HttpClient;
use opentelemetry_sdk::export::metrics::aggregation::{
    AggregationKind, Temporality, TemporalitySelector,
};
use opentelemetry_sdk::export::metrics::{AggregatorSelector, InstrumentationLibraryReader};
use opentelemetry_sdk::metrics::controllers::BasicController;
use opentelemetry_sdk::metrics::sdk_api::Descriptor;
use opentelemetry_sdk::metrics::{controllers, processors};
use opentelemetry_sdk::runtime::Runtime;
use opentelemetry_sdk::{export::metrics, Resource};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter, Write};
use std::time;

#[cfg(any(feature = "rt-tokio", feature = "rt-async-std"))]
use std::sync::{Arc, Mutex};

/// The default Dynatrace OneAgent endpoint.
const DEFAULT_ONEAGENT_ENDPOINT: &str = "http://localhost:14499/metrics/ingest";

/// The default user agent string.
const DEFAULT_USER_AGENT: &str = "opentelemetry-metric-rust";

impl DynatracePipelineBuilder {
    /// Create a Dynatrace metrics pipeline.
    pub fn metrics<AS, TS, RT>(
        self,
        aggregator_selector: AS,
        temporality_selector: TS,
        rt: RT,
    ) -> DynatraceMetricsPipeline<AS, TS, RT>
    where
        AS: AggregatorSelector + Send + Sync,
        TS: TemporalitySelector + Clone + Send + Sync,
        RT: Runtime,
    {
        DynatraceMetricsPipeline {
            rt,
            aggregator_selector,
            temporality_selector,
            exporter_pipeline: None,
            resource: None,
            period: None,
            timeout: None,
            prefix: None,
            default_dimensions: None,
            timestamp: true,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct MetricsExporterBuilder {
    builder: DynatraceExporterBuilder,
}

impl MetricsExporterBuilder {
    /// Build a Dynatrace metrics exporter with given configuration.
    fn build_metrics_exporter<TS>(
        self,
        temporality_selector: TS,
        prefix: Option<String>,
        default_dimensions: Option<DimensionSet>,
        timestamp: bool,
    ) -> Result<MetricsExporter>
    where
        TS: TemporalitySelector + Clone + Sync + Send + 'static,
    {
        MetricsExporter::new::<TS>(
            self.builder.export_config,
            self.builder.http_config.client.unwrap(),
            self.builder.http_config.headers,
            prefix,
            default_dimensions,
            timestamp,
            temporality_selector,
        )
    }
}

impl From<DynatraceExporterBuilder> for MetricsExporterBuilder {
    fn from(exporter: DynatraceExporterBuilder) -> Self {
        MetricsExporterBuilder { builder: exporter }
    }
}

/// Pipeline to build Dynatrace metrics exporter.
#[derive(Debug)]
pub struct DynatraceMetricsPipeline<AS, TS, RT>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    TS: TemporalitySelector + Clone + Send + Sync + 'static,
    RT: Runtime,
{
    rt: RT,
    aggregator_selector: AS,
    temporality_selector: TS,
    exporter_pipeline: Option<MetricsExporterBuilder>,
    resource: Option<Resource>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
    prefix: Option<String>,
    default_dimensions: Option<DimensionSet>,
    timestamp: bool,
}

impl<AS, TS, RT> DynatraceMetricsPipeline<AS, TS, RT>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    TS: TemporalitySelector + Clone + Send + Sync + 'static,
    RT: Runtime,
{
    /// Build with resource
    pub fn with_resource(self, resource: Resource) -> Self {
        DynatraceMetricsPipeline {
            resource: Some(resource),
            ..self
        }
    }

    /// Build with an exporter.
    pub fn with_exporter<B: Into<MetricsExporterBuilder>>(self, pipeline: B) -> Self {
        DynatraceMetricsPipeline {
            exporter_pipeline: Some(pipeline.into()),
            ..self
        }
    }

    /// Build with a timeout.
    pub fn with_timeout(self, timeout: time::Duration) -> Self {
        DynatraceMetricsPipeline {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Set the frequency in which metric data is exported.
    pub fn with_period(self, period: time::Duration) -> Self {
        DynatraceMetricsPipeline {
            period: Some(period),
            ..self
        }
    }

    /// Set the prefix to prepend to all metric data.
    pub fn with_prefix(self, prefix: String) -> Self {
        DynatraceMetricsPipeline {
            prefix: Some(prefix),
            ..self
        }
    }

    /// Set default dimensions to all metric data.
    pub fn with_default_dimensions(self, default_dimensions: DimensionSet) -> Self {
        DynatraceMetricsPipeline {
            default_dimensions: Some(default_dimensions),
            ..self
        }
    }

    /// Set the timestamp to all metric data.
    /// If disabled, the ingestion time of the Dynatrace server will be used automatically.
    /// Adding timestamps should be disabled in environments, where the system time is unreliable.
    pub fn with_timestamp(self, value: bool) -> Self {
        DynatraceMetricsPipeline {
            timestamp: value,
            ..self
        }
    }

    /// Build the push controller.
    pub fn build(self) -> Result<BasicController> {
        let exporter = self
            .exporter_pipeline
            .ok_or(Error::NoExporterBuilder)?
            .build_metrics_exporter(
                self.temporality_selector.clone(),
                self.prefix,
                self.default_dimensions,
                self.timestamp,
            )?;

        let mut builder = controllers::basic(processors::factory(
            self.aggregator_selector,
            self.temporality_selector,
        ))
        .with_exporter(exporter);

        if let Some(period) = self.period {
            builder = builder.with_collect_period(period);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_collect_timeout(timeout)
        }
        if let Some(resource) = self.resource {
            builder = builder.with_resource(resource);
        }
        let controller = builder.build();
        controller.start(&Context::current(), self.rt)?;

        global::set_meter_provider(controller.clone());

        Ok(controller)
    }
}

enum ClientMessage {
    Export(Box<http::Request<Vec<u8>>>),
    Shutdown,
}

/// Dynatrace metrics exporter.
pub struct MetricsExporter {
    #[cfg(feature = "rt-tokio")]
    sender: Arc<Mutex<tokio::sync::mpsc::Sender<ClientMessage>>>,

    #[cfg(all(not(feature = "rt-tokio"), feature = "rt-async-std"))]
    sender: Arc<Mutex<futures::channel::mpsc::Sender<ClientMessage>>>,

    endpoint: Uri,

    token: Option<String>,

    headers: Option<HashMap<String, String>>,

    prefix: Option<String>,

    default_dimensions: Option<DimensionSet>,

    timestamp: bool,

    temporality_selector: Arc<dyn TemporalitySelector + Send + Sync>,
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dynatrace Metrics Exporter").finish()
    }
}

impl MetricsExporter {
    /// Create a new `MetricsExporter`.
    pub fn new<T: TemporalitySelector + Clone + Send + Sync + 'static>(
        export_config: ExportConfig,
        client: Box<dyn HttpClient + 'static>,
        headers: Option<HashMap<String, String>>,
        prefix: Option<String>,
        default_dimensions: Option<DimensionSet>,
        timestamp: bool,
        temporality_selector: T,
    ) -> Result<MetricsExporter> {
        let uri: Uri = if let Some(endpoint) = export_config.endpoint {
            endpoint.parse()
        } else {
            DEFAULT_ONEAGENT_ENDPOINT.parse()
        }
        .map_err::<crate::Error, _>(Into::into)?;

        #[cfg(feature = "rt-tokio")]
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ClientMessage>(2);

        #[cfg(feature = "rt-tokio")]
        tokio::spawn(Box::pin(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    ClientMessage::Export(req) => {
                        let _ = client.send(*req).await;
                    }
                    ClientMessage::Shutdown => {
                        break;
                    }
                }
            }
        }));

        #[cfg(all(not(feature = "rt-tokio"), feature = "rt-async-std"))]
        let (sender, mut receiver) = futures::channel::mpsc::channel::<ClientMessage>(2);

        #[cfg(all(not(feature = "rt-tokio"), feature = "rt-async-std"))]
        async_std::task::spawn(Box::pin(async move {
            loop {
                match receiver.try_next() {
                    Err(_) => break,
                    Ok(result) => match result {
                        None => continue,
                        Some(msg) => match msg {
                            ClientMessage::Export(req) => {
                                let _ = client.send(*req).await;
                            }
                            ClientMessage::Shutdown => break,
                        },
                    },
                }
            }
        }));

        Ok(MetricsExporter {
            sender: Arc::new(Mutex::new(sender)),
            endpoint: uri,
            token: export_config.token,
            headers,
            prefix,
            default_dimensions,
            timestamp,
            temporality_selector: Arc::new(temporality_selector),
        })
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality_for(&self, descriptor: &Descriptor, kind: &AggregationKind) -> Temporality {
        self.temporality_selector.temporality_for(descriptor, kind)
    }
}

impl metrics::MetricsExporter for MetricsExporter {
    /// Export metric data to Dynatrace
    ///
    fn export(
        &self,
        _cx: &Context,
        _res: &Resource,
        reader: &dyn InstrumentationLibraryReader,
    ) -> Result<()> {
        let mut metric_line_data: Vec<MetricLine> = Vec::default();
        reader.try_for_each(&mut |_lib, reader| {
            reader.try_for_each(self, &mut |record| match record_to_metric_line(
                record,
                self.temporality_selector.as_ref(),
                self.prefix.clone(),
                self.default_dimensions.clone(),
                self.timestamp,
            ) {
                Ok(metric_line) => {
                    metric_line_data.extend(metric_line);
                    Ok(())
                }
                Err(err) => Err(err),
            })
        })?;

        if metric_line_data.is_empty() {
            Ok(())
        } else {
            metric_line_data
                // Send chunks of 1000 metric line data elements
                .chunks(1000)
                .try_for_each(|metric_line_data| {
                    // Transform the metric line data elements to strings
                    let metric_lines = metric_line_data
                        .iter()
                        .enumerate()
                        .fold(String::new(), |mut acc, (idx, value)| {
                            let offset = acc.len();
                            if idx > 0 {
                                acc.push('\n');
                            }

                            if write!(acc, "{}", value).is_err() {
                                acc.truncate(offset);
                            }

                            acc
                        })
                        .as_bytes()
                        .to_vec();

                    // Create a new http request
                    let mut req = http::Request::builder()
                        .method(Method::POST)
                        .uri(self.endpoint.clone())
                        .header(CONTENT_TYPE, "text/plain")
                        .header(USER_AGENT, DEFAULT_USER_AGENT)
                        .version(Version::HTTP_11)
                        .body(metric_lines)
                        .map_err::<Error, _>(Into::into)?;

                    if let Some(token) = self.token.clone() {
                        let token = format!("Api-Token {}", token);

                        let value =
                            HeaderValue::from_str(&token).map_err::<crate::Error, _>(Into::into)?;
                        req.headers_mut().insert(AUTHORIZATION, value);
                    }

                    if let Some(headers) = self.headers.clone() {
                        for (key, value) in headers {
                            let key = HeaderName::try_from(&key)
                                .map_err::<crate::Error, _>(Into::into)?;
                            let value = HeaderValue::from_str(value.as_ref())
                                .map_err::<crate::Error, _>(Into::into)?;
                            req.headers_mut().insert(key, value);
                        }
                    }

                    #[cfg(feature = "rt-tokio")]
                    self.sender
                        .lock()
                        .map(|sender| {
                            let _ = sender.try_send(ClientMessage::Export(Box::new(req)));
                        })
                        .map_err(|_| Error::PoisonedLock("dynatrace metrics exporter"))?;

                    #[cfg(all(not(feature = "rt-tokio"), feature = "rt-async-std"))]
                    self.sender
                        .lock()
                        .map(|mut sender| {
                            let _ = sender.try_send(ClientMessage::Export(Box::new(req)));
                        })
                        .map_err(|_| Error::PoisonedLock("dynatrace metrics exporter"))?;

                    Ok(())
                })
        }
    }
}

impl Drop for MetricsExporter {
    fn drop(&mut self) {
        #[cfg(feature = "rt-tokio")]
        let _sender_lock_guard = self.sender.lock().map(|sender| {
            let _ = sender.try_send(ClientMessage::Shutdown);
        });

        #[cfg(all(not(feature = "rt-tokio"), feature = "rt-async-std"))]
        let _sender_lock_guard = self.sender.lock().map(|mut sender| {
            let _ = sender.try_send(ClientMessage::Shutdown);
        });
    }
}
