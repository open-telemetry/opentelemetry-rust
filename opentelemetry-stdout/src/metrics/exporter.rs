use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::{f64, fmt};
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{
            ExponentialHistogram, Gauge, GaugeDataPoint, Histogram, HistogramDataPoint,
            ResourceMetrics, ScopeMetrics, Sum, SumDataPoint,
        },
        exporter::PushMetricExporter,
    },
};
use std::fmt::Debug;
use std::sync::atomic;

/// An OpenTelemetry exporter that writes to stdout on export.
pub struct MetricExporter {
    is_shutdown: atomic::AtomicBool,
    temporality: Temporality,
}

impl MetricExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> MetricExporterBuilder {
        MetricExporterBuilder::default()
    }
}
impl Default for MetricExporter {
    fn default() -> Self {
        MetricExporterBuilder::default().build()
    }
}

impl fmt::Debug for MetricExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricExporter")
    }
}

#[async_trait]
impl PushMetricExporter for MetricExporter {
    /// Write Metrics to stdout
    async fn export(&self, metrics: &mut ResourceMetrics) -> OTelSdkResult {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Err(opentelemetry_sdk::error::OTelSdkError::AlreadyShutdown)
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

    async fn force_flush(&self) -> OTelSdkResult {
        // exporter holds no state, nothing to flush
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

fn print_metrics(metrics: &[ScopeMetrics]) {
    for (i, metric) in metrics.iter().enumerate() {
        println!("\tInstrumentation Scope #{}", i);
        println!("\t\tName         : {}", &metric.scope.name());
        if let Some(version) = &metric.scope.version() {
            println!("\t\tVersion  : {:?}", version);
        }
        if let Some(schema_url) = &metric.scope.schema_url() {
            println!("\t\tSchemaUrl: {:?}", schema_url);
        }
        metric
            .scope
            .attributes()
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
            if let Some(hist) = data.downcast_ref::<Histogram<u64>>() {
                println!("\t\tType         : Histogram");
                print_histogram(hist);
            } else if let Some(hist) = data.downcast_ref::<Histogram<f64>>() {
                println!("\t\tType         : Histogram");
                print_histogram(hist);
            } else if let Some(_hist) = data.downcast_ref::<ExponentialHistogram<u64>>() {
                println!("\t\tType         : Exponential Histogram");
                // TODO
            } else if let Some(_hist) = data.downcast_ref::<ExponentialHistogram<f64>>() {
                println!("\t\tType         : Exponential Histogram");
                // TODO
            } else if let Some(sum) = data.downcast_ref::<Sum<u64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(sum) = data.downcast_ref::<Sum<i64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(sum) = data.downcast_ref::<Sum<f64>>() {
                println!("\t\tType         : Sum");
                print_sum(sum);
            } else if let Some(gauge) = data.downcast_ref::<Gauge<u64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else if let Some(gauge) = data.downcast_ref::<Gauge<i64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else if let Some(gauge) = data.downcast_ref::<Gauge<f64>>() {
                println!("\t\tType         : Gauge");
                print_gauge(gauge);
            } else {
                println!("Unsupported data type");
            }
        });
    }
}

fn print_sum<T: Debug>(sum: &Sum<T>) {
    println!("\t\tSum DataPoints");
    println!("\t\tMonotonic    : {}", sum.is_monotonic);
    if sum.temporality == Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    let datetime: DateTime<Utc> = sum.start_time.into();
    println!(
        "\t\tStartTime    : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    let datetime: DateTime<Utc> = sum.time.into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    print_sum_data_points(&sum.data_points);
}

fn print_gauge<T: Debug>(gauge: &Gauge<T>) {
    println!("\t\tGauge DataPoints");
    if let Some(start_time) = gauge.start_time {
        let datetime: DateTime<Utc> = start_time.into();
        println!(
            "\t\tStartTime    : {}",
            datetime.format("%Y-%m-%d %H:%M:%S%.6f")
        );
    }
    let datetime: DateTime<Utc> = gauge.time.into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    print_gauge_data_points(&gauge.data_points);
}

fn print_histogram<T: Debug>(histogram: &Histogram<T>) {
    if histogram.temporality == Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    let datetime: DateTime<Utc> = histogram.start_time.into();
    println!(
        "\t\tStartTime    : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    let datetime: DateTime<Utc> = histogram.time.into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    println!("\t\tHistogram DataPoints");
    print_hist_data_points(&histogram.data_points);
}

fn print_sum_data_points<T: Debug>(data_points: &[SumDataPoint<T>]) {
    for (i, data_point) in data_points.iter().enumerate() {
        println!("\t\tDataPoint #{}", i);
        println!("\t\t\tValue        : {:#?}", data_point.value);
        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes.iter() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

fn print_gauge_data_points<T: Debug>(data_points: &[GaugeDataPoint<T>]) {
    for (i, data_point) in data_points.iter().enumerate() {
        println!("\t\tDataPoint #{}", i);
        println!("\t\t\tValue        : {:#?}", data_point.value);
        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes.iter() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

fn print_hist_data_points<T: Debug>(data_points: &[HistogramDataPoint<T>]) {
    for (i, data_point) in data_points.iter().enumerate() {
        println!("\t\tDataPoint #{}", i);
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

        println!("\t\t\tBuckets");
        let mut lower_bound = f64::NEG_INFINITY;
        for (i, &upper_bound) in data_point.bounds.iter().enumerate() {
            let count = data_point.bucket_counts.get(i).unwrap_or(&0);
            println!("\t\t\t\t {} to {} : {}", lower_bound, upper_bound, count);
            lower_bound = upper_bound;
        }

        let last_count = data_point
            .bucket_counts
            .get(data_point.bounds.len())
            .unwrap_or(&0);
        println!("\t\t\t\t{} to +Infinity : {}", lower_bound, last_count);
    }
}

/// Configuration for the stdout metrics exporter
#[derive(Default)]
pub struct MetricExporterBuilder {
    temporality: Option<Temporality>,
}

impl MetricExporterBuilder {
    /// Set the [Temporality] of the exporter.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Create a metrics exporter with the current configuration
    pub fn build(self) -> MetricExporter {
        MetricExporter {
            temporality: self.temporality.unwrap_or_default(),
            is_shutdown: atomic::AtomicBool::new(false),
        }
    }
}

impl fmt::Debug for MetricExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricExporterBuilder")
    }
}
