use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::{f64, fmt};
use opentelemetry::metrics::{MetricsError, Result};
use opentelemetry_sdk::metrics::{
    data::{self, ScopeMetrics},
    exporter::PushMetricsExporter,
    reader::{
        AggregationSelector, DefaultAggregationSelector, DefaultTemporalitySelector,
        TemporalitySelector,
    },
    Aggregation, InstrumentKind,
};
use std::fmt::Debug;
use std::sync::atomic;

/// An OpenTelemetry exporter that writes to stdout on export.
pub struct MetricsExporter {
    is_shutdown: atomic::AtomicBool,
    temporality_selector: Box<dyn TemporalitySelector>,
    aggregation_selector: Box<dyn AggregationSelector>,
}

impl MetricsExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> MetricsExporterBuilder {
        MetricsExporterBuilder::default()
    }
}
impl Default for MetricsExporter {
    fn default() -> Self {
        MetricsExporterBuilder::default().build()
    }
}

impl fmt::Debug for MetricsExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricsExporter")
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality(&self, kind: InstrumentKind) -> data::Temporality {
        self.temporality_selector.temporality(kind)
    }
}

impl AggregationSelector for MetricsExporter {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.aggregation_selector.aggregation(kind)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl PushMetricsExporter for MetricsExporter {
    /// Write Metrics to stdout
    async fn export(&self, metrics: &mut data::ResourceMetrics) -> Result<()> {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Err(MetricsError::Other("exporter is shut down".into()))
        } else {
            println!("Metrics");
            println!("Resource");
            if let Some(schema_url) = metrics.resource.schema_url() {
                println!("\tResource SchemaUrl: {:?}", schema_url);
            }

            metrics.resource.iter().for_each(|(k, v)| {
                println!("\t ->  {}={:?}", k, v);
            });
            print_metrics(&metrics.scope_metrics);
            Ok(())
        }
    }

    async fn force_flush(&self) -> Result<()> {
        // exporter holds no state, nothing to flush
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }
}

fn print_metrics(metrics: &[ScopeMetrics]) {
    for (i, metric) in metrics.iter().enumerate() {
        println!("\tInstrumentation Scope #{}", i);
        println!("\t\tName         : {}", &metric.scope.name);
        if let Some(version) = &metric.scope.version {
            println!("\t\tVersion  : {:?}", version);
        }
        if let Some(schema_url) = &metric.scope.schema_url {
            println!("\t\tSchemaUrl: {:?}", schema_url);
        }
        metric
            .scope
            .attributes
            .iter()
            .enumerate()
            .for_each(|(index, kv)| {
                if index == 0 {
                    println!("\t\tScope Attributes:");
                }
                println!("\t\t\t ->  {}: {}", kv.key, kv.value);
            });

        metric.metrics.iter().enumerate().for_each(|(i, metric)| {
            println!("Metric #{}", i);
            println!("\t\tName         : {}", &metric.name);
            println!("\t\tDescription  : {}", &metric.description);
            println!("\t\tUnit         : {}", &metric.unit);

            let data = metric.data.as_any();
            if let Some(hist) = data.downcast_ref::<data::Histogram<u64>>() {
                println!("\t\tType         : Histogram");
                print_histogram(hist);
            } else if let Some(hist) = data.downcast_ref::<data::Histogram<f64>>() {
                println!("\t\tType         : Histogram");
                print_histogram(hist);
            } else if let Some(_hist) = data.downcast_ref::<data::ExponentialHistogram<u64>>() {
                println!("\t\tType         : Exponential Histogram");
                // TODO
            } else if let Some(_hist) = data.downcast_ref::<data::ExponentialHistogram<f64>>() {
                println!("\t\tType         : Exponential Histogram");
                // TODO
            } else if let Some(sum) = data.downcast_ref::<data::Sum<u64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(sum) = data.downcast_ref::<data::Sum<i64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(sum) = data.downcast_ref::<data::Sum<f64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(gauge) = data.downcast_ref::<data::Gauge<u64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else if let Some(gauge) = data.downcast_ref::<data::Gauge<i64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else if let Some(gauge) = data.downcast_ref::<data::Gauge<f64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else {
                println!("Unsupported data type");
            }
        });
    }
}

fn print_sum<T: Debug>(sum: &data::Sum<T>) {
    println!("\t\tSum DataPoints");
    println!("\t\tMonotonic    : {}", sum.is_monotonic);
    if sum.temporality == data::Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    print_data_points(&sum.data_points);
}

fn print_gauge<T: Debug>(gauge: &data::Gauge<T>) {
    println!("\t\tGauge DataPoints");
    print_data_points(&gauge.data_points);
}

fn print_histogram<T: Debug>(histogram: &data::Histogram<T>) {
    if histogram.temporality == data::Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    println!("\t\tHistogram DataPoints");
    print_hist_data_points(&histogram.data_points);
}

fn print_data_points<T: Debug>(data_points: &[data::DataPoint<T>]) {
    for (i, data_point) in data_points.iter().enumerate() {
        println!("\t\tDataPoint #{}", i);
        if let Some(start_time) = data_point.start_time {
            let datetime: DateTime<Utc> = start_time.into();
            println!(
                "\t\t\tStartTime    : {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        if let Some(end_time) = data_point.time {
            let datetime: DateTime<Utc> = end_time.into();
            println!(
                "\t\t\tEndTime      : {}",
                datetime.format("%Y-%m-%d %H:%M:%S%.6f")
            );
        }
        println!("\t\t\tValue        : {:#?}", data_point.value);
        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes.iter() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

fn print_hist_data_points<T: Debug>(data_points: &[data::HistogramDataPoint<T>]) {
    for (i, data_point) in data_points.iter().enumerate() {
        println!("\t\tDataPoint #{}", i);
        let datetime: DateTime<Utc> = data_point.start_time.into();
        println!(
            "\t\t\tStartTime    : {}",
            datetime.format("%Y-%m-%d %H:%M:%S%.6f")
        );
        let datetime: DateTime<Utc> = data_point.time.into();
        println!(
            "\t\t\tEndTime      : {}",
            datetime.format("%Y-%m-%d %H:%M:%S%.6f")
        );
        println!("\t\t\tCount        : {}", data_point.count);
        println!("\t\t\tSum          : {:?}", data_point.sum);
        if let Some(min) = &data_point.min {
            println!("\t\t\tMin          : {:?}", min);
        }

        if let Some(max) = &data_point.max {
            println!("\t\t\tMax          : {:?}", max);
        }

        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes.iter() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

/// Configuration for the stdout metrics exporter
#[derive(Default)]
pub struct MetricsExporterBuilder {
    temporality_selector: Option<Box<dyn TemporalitySelector>>,
    aggregation_selector: Option<Box<dyn AggregationSelector>>,
}

impl MetricsExporterBuilder {
    /// Set the temporality exporter for the exporter
    pub fn with_temporality_selector(
        mut self,
        selector: impl TemporalitySelector + 'static,
    ) -> Self {
        self.temporality_selector = Some(Box::new(selector));
        self
    }

    /// Set the aggregation exporter for the exporter
    pub fn with_aggregation_selector(
        mut self,
        selector: impl AggregationSelector + 'static,
    ) -> Self {
        self.aggregation_selector = Some(Box::new(selector));
        self
    }

    /// Create a metrics exporter with the current configuration
    pub fn build(self) -> MetricsExporter {
        MetricsExporter {
            temporality_selector: self
                .temporality_selector
                .unwrap_or_else(|| Box::new(DefaultTemporalitySelector::new())),
            aggregation_selector: self
                .aggregation_selector
                .unwrap_or_else(|| Box::new(DefaultAggregationSelector::new())),
            is_shutdown: atomic::AtomicBool::new(false),
        }
    }
}

impl fmt::Debug for MetricsExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricsExporterBuilder")
    }
}
