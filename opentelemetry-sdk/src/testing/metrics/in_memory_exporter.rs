use crate::metrics::data::{Histogram, Metric, ResourceMetrics, ScopeMetrics, Temporality};
use crate::metrics::exporter::PushMetricsExporter;
use crate::metrics::reader::{
    AggregationSelector, DefaultAggregationSelector, DefaultTemporalitySelector,
    TemporalitySelector,
};
use crate::metrics::{data, Aggregation, InstrumentKind};
use async_trait::async_trait;
use opentelemetry::metrics::MetricsError;
use opentelemetry::metrics::Result;
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};

/// An in-memory metrics exporter that stores metrics data in memory.
///
/// This exporter is useful for testing and debugging purposes. It stores
/// metric data in a `VecDeque<ResourceMetrics>`. Metrics can be retrieved
/// using the `get_finished_metrics` method.
///
/// # Panics
///
/// This exporter may panic
/// - if there's an issue with locking the `metrics` Mutex, such as if the Mutex is poisoned.
/// - the data point recorded is not one of [i64, u64, f64]. This shouldn't happen if used with OpenTelemetry API.
///
/// # Example
///
/// ```
///# use opentelemetry_sdk::{metrics, runtime};
///# use opentelemetry::{KeyValue};
///# use opentelemetry::attributes::AttributeSet;
///# use opentelemetry::metrics::MeterProvider;
///# use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
///# use opentelemetry_sdk::metrics::PeriodicReader;
///
///# #[tokio::main]
///# async fn main() {
/// // Create an InMemoryMetricsExporter
///  let exporter = InMemoryMetricsExporter::default();
///
///  // Create a MeterProvider and register the exporter
///  let meter_provider = metrics::SdkMeterProvider::builder()
///      .with_reader(PeriodicReader::builder(exporter.clone(), runtime::Tokio).build())
///      .build();
///
///  // Create and record metrics using the MeterProvider
///  let meter = meter_provider.meter(std::borrow::Cow::Borrowed("example"));
///  let counter = meter.u64_counter("my_counter").init();
///  let attributes = AttributeSet::from([KeyValue::new("key", "value")]);
///  counter.add(1, attributes);
///
///  meter_provider.force_flush().unwrap();
///
///  // Retrieve the finished metrics from the exporter
///  let finished_metrics = exporter.get_finished_metrics().unwrap();
///
///  // Print the finished metrics
/// for resource_metrics in finished_metrics {
///      println!("{:?}", resource_metrics);
///  }
///# }
/// ```
pub struct InMemoryMetricsExporter {
    metrics: Arc<Mutex<VecDeque<ResourceMetrics>>>,
    aggregation_selector: Arc<dyn AggregationSelector + Send + Sync>,
    temporality_selector: Arc<dyn TemporalitySelector + Send + Sync>,
}

impl Clone for InMemoryMetricsExporter {
    fn clone(&self) -> Self {
        InMemoryMetricsExporter {
            metrics: self.metrics.clone(),
            aggregation_selector: self.aggregation_selector.clone(),
            temporality_selector: self.temporality_selector.clone(),
        }
    }
}

impl fmt::Debug for InMemoryMetricsExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryMetricsExporter").finish()
    }
}

impl Default for InMemoryMetricsExporter {
    fn default() -> Self {
        InMemoryMetricsExporterBuilder::new().build()
    }
}

/// Builder for [`InMemoryMetricsExporter`].
/// # Example
///
/// ```
/// # use opentelemetry_sdk::testing::metrics::{InMemoryMetricsExporter, InMemoryMetricsExporterBuilder};
///
/// let exporter = InMemoryMetricsExporterBuilder::new().build();
/// ```
pub struct InMemoryMetricsExporterBuilder {
    aggregation_selector: Option<Arc<dyn AggregationSelector + Send + Sync>>,
    temporality_selector: Option<Arc<dyn TemporalitySelector + Send + Sync>>,
}

impl fmt::Debug for InMemoryMetricsExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryMetricsExporterBuilder").finish()
    }
}

impl Default for InMemoryMetricsExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMetricsExporterBuilder {
    /// Creates a new instance of the `InMemoryMetricsExporterBuilder`.
    pub fn new() -> Self {
        Self {
            aggregation_selector: None,
            temporality_selector: None,
        }
    }

    /// Sets the aggregation selector for the exporter.
    pub fn with_aggregation_selector<T>(mut self, aggregation_selector: T) -> Self
    where
        T: AggregationSelector + Send + Sync + 'static,
    {
        self.aggregation_selector = Some(Arc::new(aggregation_selector));
        self
    }

    /// Sets the temporality selector for the exporter.
    pub fn with_temporality_selector<T>(mut self, temporality_selector: T) -> Self
    where
        T: TemporalitySelector + Send + Sync + 'static,
    {
        self.temporality_selector = Some(Arc::new(temporality_selector));
        self
    }

    /// Creates a new instance of the `InMemoryMetricsExporter`.
    ///
    pub fn build(self) -> InMemoryMetricsExporter {
        InMemoryMetricsExporter {
            metrics: Arc::new(Mutex::new(VecDeque::new())),
            aggregation_selector: self
                .aggregation_selector
                .unwrap_or_else(|| Arc::new(DefaultAggregationSelector::default())),
            temporality_selector: self
                .temporality_selector
                .unwrap_or_else(|| Arc::new(DefaultTemporalitySelector::default())),
        }
    }
}

impl InMemoryMetricsExporter {
    /// Returns the finished metrics as a vector of `ResourceMetrics`.
    ///
    /// # Errors
    ///
    /// Returns a `MetricsError` if the internal lock cannot be acquired.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
    ///
    /// let exporter = InMemoryMetricsExporter::default();
    /// let finished_metrics = exporter.get_finished_metrics().unwrap();
    /// ```
    pub fn get_finished_metrics(&self) -> Result<Vec<ResourceMetrics>> {
        self.metrics
            .lock()
            .map(|metrics_guard| metrics_guard.iter().map(Self::clone_metrics).collect())
            .map_err(MetricsError::from)
    }

    /// Clears the internal storage of finished metrics.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
    ///
    /// let exporter = InMemoryMetricsExporter::default();
    /// exporter.reset();
    /// ```
    pub fn reset(&self) {
        let _ = self
            .metrics
            .lock()
            .map(|mut metrics_guard| metrics_guard.clear());
    }

    fn clone_metrics(metric: &ResourceMetrics) -> ResourceMetrics {
        ResourceMetrics {
            resource: metric.resource.clone(),
            scope_metrics: metric
                .scope_metrics
                .iter()
                .map(|scope_metric| ScopeMetrics {
                    scope: scope_metric.scope.clone(),
                    metrics: scope_metric
                        .metrics
                        .iter()
                        .map(|metric| Metric {
                            name: metric.name.clone(),
                            description: metric.description.clone(),
                            unit: metric.unit.clone(),
                            // we don't expect any unknown data type here
                            data: Self::clone_data(metric.data.as_ref()).unwrap(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn clone_data(data: &dyn data::Aggregation) -> Option<Box<dyn data::Aggregation>> {
        if let Some(hist) = data.as_any().downcast_ref::<Histogram<i64>>() {
            Some(Box::new(Histogram {
                data_points: hist.data_points.clone(),
                temporality: hist.temporality,
            }))
        } else if let Some(hist) = data.as_any().downcast_ref::<Histogram<f64>>() {
            Some(Box::new(Histogram {
                data_points: hist.data_points.clone(),
                temporality: hist.temporality,
            }))
        } else if let Some(hist) = data.as_any().downcast_ref::<Histogram<u64>>() {
            Some(Box::new(Histogram {
                data_points: hist.data_points.clone(),
                temporality: hist.temporality,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<data::Sum<i64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<data::Sum<f64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<data::Sum<u64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<data::Gauge<i64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<data::Gauge<f64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<data::Gauge<u64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
            }))
        } else {
            // unknown data type
            None
        }
    }
}

impl AggregationSelector for InMemoryMetricsExporter {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.aggregation_selector.aggregation(kind)
    }
}

impl TemporalitySelector for InMemoryMetricsExporter {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.temporality_selector.temporality(kind)
    }
}

#[async_trait]
impl PushMetricsExporter for InMemoryMetricsExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        self.metrics
            .lock()
            .map(|mut metrics_guard| {
                metrics_guard.push_back(InMemoryMetricsExporter::clone_metrics(metrics))
            })
            .map_err(MetricsError::from)
    }

    async fn force_flush(&self) -> Result<()> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> Result<()> {
        self.metrics
            .lock()
            .map(|mut metrics_guard| metrics_guard.clear())
            .map_err(MetricsError::from)?;

        Ok(())
    }
}
