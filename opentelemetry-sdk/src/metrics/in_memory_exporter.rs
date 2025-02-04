use crate::error::{OTelSdkError, OTelSdkResult};
use crate::metrics::data::{self, Gauge, Sum};
use crate::metrics::data::{Histogram, Metric, ResourceMetrics, ScopeMetrics};
use crate::metrics::exporter::PushMetricExporter;
use crate::metrics::MetricError;
use crate::metrics::MetricResult;
use crate::metrics::Temporality;
use async_trait::async_trait;
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
///# use opentelemetry_sdk::metrics;
///# use opentelemetry::{KeyValue};
///# use opentelemetry::metrics::MeterProvider;
///# use opentelemetry_sdk::metrics::InMemoryMetricExporter;
///# use opentelemetry_sdk::metrics::PeriodicReader;
///
///# #[tokio::main]
///# async fn main() {
/// // Create an InMemoryMetricExporter
///  let exporter = InMemoryMetricExporter::default();
///
///  // Create a MeterProvider and register the exporter
///  let meter_provider = metrics::SdkMeterProvider::builder()
///      .with_reader(PeriodicReader::builder(exporter.clone()).build())
///      .build();
///
///  // Create and record metrics using the MeterProvider
///  let meter = meter_provider.meter("example");
///  let counter = meter.u64_counter("my_counter").build();
///  counter.add(1, &[KeyValue::new("key", "value")]);
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
pub struct InMemoryMetricExporter {
    metrics: Arc<Mutex<VecDeque<ResourceMetrics>>>,
    temporality: Temporality,
}

impl Clone for InMemoryMetricExporter {
    fn clone(&self) -> Self {
        InMemoryMetricExporter {
            metrics: self.metrics.clone(),
            temporality: self.temporality,
        }
    }
}

impl fmt::Debug for InMemoryMetricExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryMetricExporter").finish()
    }
}

impl Default for InMemoryMetricExporter {
    fn default() -> Self {
        InMemoryMetricExporterBuilder::new().build()
    }
}

/// Builder for [`InMemoryMetricExporter`].
/// # Example
///
/// ```
/// # use opentelemetry_sdk::metrics::{InMemoryMetricExporter, InMemoryMetricExporterBuilder};
///
/// let exporter = InMemoryMetricExporterBuilder::new().build();
/// ```
pub struct InMemoryMetricExporterBuilder {
    temporality: Option<Temporality>,
}

impl fmt::Debug for InMemoryMetricExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryMetricExporterBuilder").finish()
    }
}

impl Default for InMemoryMetricExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMetricExporterBuilder {
    /// Creates a new instance of the `InMemoryMetricExporterBuilder`.
    pub fn new() -> Self {
        Self { temporality: None }
    }

    /// Set the [Temporality] of the exporter.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Creates a new instance of the `InMemoryMetricExporter`.
    ///
    pub fn build(self) -> InMemoryMetricExporter {
        InMemoryMetricExporter {
            metrics: Arc::new(Mutex::new(VecDeque::new())),
            temporality: self.temporality.unwrap_or_default(),
        }
    }
}

impl InMemoryMetricExporter {
    /// Returns the finished metrics as a vector of `ResourceMetrics`.
    ///
    /// # Errors
    ///
    /// Returns a `MetricError` if the internal lock cannot be acquired.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::metrics::InMemoryMetricExporter;
    ///
    /// let exporter = InMemoryMetricExporter::default();
    /// let finished_metrics = exporter.get_finished_metrics().unwrap();
    /// ```
    pub fn get_finished_metrics(&self) -> MetricResult<Vec<ResourceMetrics>> {
        self.metrics
            .lock()
            .map(|metrics_guard| metrics_guard.iter().map(Self::clone_metrics).collect())
            .map_err(MetricError::from)
    }

    /// Clears the internal storage of finished metrics.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::metrics::InMemoryMetricExporter;
    ///
    /// let exporter = InMemoryMetricExporter::default();
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
                start_time: hist.start_time,
                time: hist.time,
                temporality: hist.temporality,
            }))
        } else if let Some(hist) = data.as_any().downcast_ref::<Histogram<f64>>() {
            Some(Box::new(Histogram {
                data_points: hist.data_points.clone(),
                start_time: hist.start_time,
                time: hist.time,
                temporality: hist.temporality,
            }))
        } else if let Some(hist) = data.as_any().downcast_ref::<Histogram<u64>>() {
            Some(Box::new(Histogram {
                data_points: hist.data_points.clone(),
                start_time: hist.start_time,
                time: hist.time,
                temporality: hist.temporality,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<Sum<i64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                start_time: sum.start_time,
                time: sum.time,
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<Sum<f64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                start_time: sum.start_time,
                time: sum.time,
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(sum) = data.as_any().downcast_ref::<Sum<u64>>() {
            Some(Box::new(data::Sum {
                data_points: sum.data_points.clone(),
                start_time: sum.start_time,
                time: sum.time,
                temporality: sum.temporality,
                is_monotonic: sum.is_monotonic,
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<Gauge<i64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
                start_time: gauge.start_time,
                time: gauge.time,
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<Gauge<f64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
                start_time: gauge.start_time,
                time: gauge.time,
            }))
        } else if let Some(gauge) = data.as_any().downcast_ref::<Gauge<u64>>() {
            Some(Box::new(data::Gauge {
                data_points: gauge.data_points.clone(),
                start_time: gauge.start_time,
                time: gauge.time,
            }))
        } else {
            // unknown data type
            None
        }
    }
}

#[async_trait]
impl PushMetricExporter for InMemoryMetricExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> OTelSdkResult {
        self.metrics
            .lock()
            .map(|mut metrics_guard| {
                metrics_guard.push_back(InMemoryMetricExporter::clone_metrics(metrics))
            })
            .map_err(|_| OTelSdkError::InternalFailure("Failed to lock metrics".to_string()))
    }

    async fn force_flush(&self) -> OTelSdkResult {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> OTelSdkResult {
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}
