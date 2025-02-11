use crate::error::{OTelSdkError, OTelSdkResult};
use crate::metrics::data::{self, Gauge, Sum};
use crate::metrics::data::{Histogram, Metric, ResourceMetrics, ScopeMetrics};
use crate::metrics::exporter::PushMetricExporter;
use crate::metrics::Temporality;
use async_trait::async_trait;
use std::fmt;
use std::sync::{Arc, Mutex};

/// An in-memory metrics exporter that stores metrics data in memory.
///
/// This exporter is useful for testing and debugging purposes. It stores
/// metric data in a user provided Vec.
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
///  let exported_metrics =  Arc::new(Mutex::new(Vec::new()));
///  let exporter = InMemoryMetricExporter::builder().with_metrics(exported_metrics.clone()).build();
///
///  // Create a MeterProvider and register the exporter
///  let meter_provider = metrics::SdkMeterProvider::builder()
///      .with_reader(PeriodicReader::builder(exporter).build())
///      .build();
///
///  // Create and record metrics using the MeterProvider
///  let meter = meter_provider.meter("example");
///  let counter = meter.u64_counter("my_counter").build();
///  counter.add(1, &[KeyValue::new("key", "value")]);
///
///  meter_provider.force_flush().unwrap();
///
///
///  // Print the finished metrics
/// for resource_metrics in exported_metrics.lock().unwrap().iter() {
///      println!("{:?}", resource_metrics);
///  }
///# }
/// ```
pub struct InMemoryMetricExporter {
    metrics: Arc<Mutex<Vec<ResourceMetrics>>>,
    temporality: Temporality,
}

impl fmt::Debug for InMemoryMetricExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryMetricExporter").finish()
    }
}

impl InMemoryMetricExporter {
    /// Creates a builder.
    pub fn builder() -> InMemoryMetricExporterBuilder {
        InMemoryMetricExporterBuilder::new()
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
    metrics: Option<Arc<Mutex<Vec<ResourceMetrics>>>>,
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
        Self { temporality: None,
               metrics: None }
    }

    /// Set the [Temporality] of the exporter.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Set the collection to store the metrics.
    pub fn with_metrics(mut self, metrics: Arc<Mutex<Vec<ResourceMetrics>>>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Creates a new instance of the `InMemoryMetricExporter`.
    ///
    pub fn build(self) -> InMemoryMetricExporter {
        InMemoryMetricExporter {
            metrics: self.metrics.expect("Metric collection is required"),
            temporality: self.temporality.unwrap_or_default(),
        }
    }
}

impl InMemoryMetricExporter {
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
                metrics_guard.push(InMemoryMetricExporter::clone_metrics(metrics))
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
