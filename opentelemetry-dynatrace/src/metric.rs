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
use futures::Stream;
use http::{
    header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Method, Uri, Version,
};
use opentelemetry::metrics::{Descriptor, Result};
use opentelemetry::sdk::export::metrics::{AggregatorSelector, ExportKindSelector};
use opentelemetry::sdk::metrics::{PushController, PushControllerWorker};
use opentelemetry::sdk::{
    export::metrics::{CheckpointSet, ExportKind, ExportKindFor, Exporter},
    metrics::selectors,
    Resource,
};
use opentelemetry::{global, KeyValue};
use opentelemetry_http::HttpClient;
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
    pub fn metrics<SP, SO, I, IO>(
        self,
        spawn: SP,
        interval: I,
    ) -> DynatraceMetricsPipeline<selectors::simple::Selector, ExportKindSelector, SP, SO, I, IO>
    where
        SP: Fn(PushControllerWorker) -> SO,
        I: Fn(time::Duration) -> IO,
    {
        DynatraceMetricsPipeline {
            aggregator_selector: selectors::simple::Selector::Inexpensive,
            export_selector: ExportKindSelector::Cumulative,
            spawn,
            interval,
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
    fn build_metrics_exporter<ES>(
        self,
        export_selector: ES,
        prefix: Option<String>,
        default_dimensions: Option<DimensionSet>,
        timestamp: bool,
    ) -> Result<MetricsExporter>
    where
        ES: ExportKindFor + Sync + Send + 'static,
    {
        MetricsExporter::new::<ES>(
            self.builder.export_config,
            self.builder.http_config.client.unwrap(),
            self.builder.http_config.headers,
            prefix,
            default_dimensions,
            timestamp,
            export_selector,
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
pub struct DynatraceMetricsPipeline<AS, ES, SP, SO, I, IO>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    ES: ExportKindFor + Send + Sync + Clone + 'static,
    SP: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IO,
{
    aggregator_selector: AS,
    export_selector: ES,
    spawn: SP,
    interval: I,
    exporter_pipeline: Option<MetricsExporterBuilder>,
    resource: Option<Resource>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
    prefix: Option<String>,
    default_dimensions: Option<DimensionSet>,
    timestamp: bool,
}

impl<AS, ES, SP, SO, I, IO, IOI> DynatraceMetricsPipeline<AS, ES, SP, SO, I, IO>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    ES: ExportKindFor + Send + Sync + Clone + 'static,
    SP: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IO,
    IO: Stream<Item = IOI> + Send + 'static,
{
    /// Build with resource key value pairs.
    pub fn with_resource<T: IntoIterator<Item = R>, R: Into<KeyValue>>(self, resource: T) -> Self {
        DynatraceMetricsPipeline {
            resource: Some(Resource::new(resource.into_iter().map(Into::into))),
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

    /// Build with an aggregator selector.
    pub fn with_aggregator_selector<T>(
        self,
        aggregator_selector: T,
    ) -> DynatraceMetricsPipeline<T, ES, SP, SO, I, IO>
    where
        T: AggregatorSelector + Send + Sync + 'static,
    {
        DynatraceMetricsPipeline {
            aggregator_selector,
            export_selector: self.export_selector,
            spawn: self.spawn,
            interval: self.interval,
            exporter_pipeline: self.exporter_pipeline,
            resource: self.resource,
            period: self.period,
            timeout: self.timeout,
            prefix: self.prefix,
            default_dimensions: self.default_dimensions,
            timestamp: self.timestamp,
        }
    }

    /// Build with a spawn function.
    pub fn with_spawn(self, spawn: SP) -> Self {
        DynatraceMetricsPipeline { spawn, ..self }
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

    /// Build with an interval function.
    pub fn with_interval(self, interval: I) -> Self {
        DynatraceMetricsPipeline { interval, ..self }
    }

    /// Build with an export kind selector.
    pub fn with_export_kind<E>(
        self,
        export_selector: E,
    ) -> DynatraceMetricsPipeline<AS, E, SP, SO, I, IO>
    where
        E: ExportKindFor + Send + Sync + Clone + 'static,
    {
        DynatraceMetricsPipeline {
            aggregator_selector: self.aggregator_selector,
            export_selector,
            spawn: self.spawn,
            interval: self.interval,
            exporter_pipeline: self.exporter_pipeline,
            resource: self.resource,
            period: self.period,
            timeout: self.timeout,
            prefix: self.prefix,
            default_dimensions: self.default_dimensions,
            timestamp: self.timestamp,
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
    pub fn build(self) -> Result<PushController> {
        let exporter = self
            .exporter_pipeline
            .ok_or(Error::NoExporterBuilder)?
            .build_metrics_exporter(
                self.export_selector.clone(),
                self.prefix,
                self.default_dimensions,
                self.timestamp,
            )?;

        let mut builder = opentelemetry::sdk::metrics::controllers::push(
            self.aggregator_selector,
            self.export_selector,
            exporter,
            self.spawn,
            self.interval,
        );
        if let Some(period) = self.period {
            builder = builder.with_period(period);
        }
        if let Some(resource) = self.resource {
            builder = builder.with_resource(resource);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout)
        }
        let controller = builder.build();
        global::set_meter_provider(controller.provider());
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

    export_kind_selector: Arc<dyn ExportKindFor + Send + Sync>,
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dynatrace Metrics Exporter").finish()
    }
}

impl ExportKindFor for MetricsExporter {
    fn export_kind_for(&self, descriptor: &Descriptor) -> ExportKind {
        self.export_kind_selector.export_kind_for(descriptor)
    }
}

impl MetricsExporter {
    /// Create a new `MetricsExporter`.
    pub fn new<T: ExportKindFor + Send + Sync + 'static>(
        export_config: ExportConfig,
        client: Box<dyn HttpClient + 'static>,
        headers: Option<HashMap<String, String>>,
        prefix: Option<String>,
        default_dimensions: Option<DimensionSet>,
        timestamp: bool,
        export_kind_selector: T,
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
            export_kind_selector: Arc::new(export_kind_selector),
        })
    }
}

impl Exporter for MetricsExporter {
    /// Export metric data to Dynatrace
    ///
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()> {
        let mut metric_line_data: Vec<MetricLine> = Vec::default();
        checkpoint_set.try_for_each(self.export_kind_selector.as_ref(), &mut |record| {
            match record_to_metric_line(
                record,
                self.export_kind_selector.as_ref(),
                self.prefix.clone(),
                self.default_dimensions.clone(),
                self.timestamp,
            ) {
                Ok(metric_line) => {
                    metric_line_data.extend(metric_line);
                    Ok(())
                }
                Err(err) => Err(err),
            }
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
