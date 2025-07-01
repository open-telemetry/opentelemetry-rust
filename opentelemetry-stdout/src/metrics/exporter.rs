use chrono::{DateTime, Utc};
use core::{f64, fmt};
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{
            Gauge, GaugeDataPoint, Histogram, HistogramDataPoint, ResourceMetrics, ScopeMetrics,
            Sum, SumDataPoint,
        },
        exporter::PushMetricExporter,
    },
};
use std::fmt::Debug;
use std::sync::atomic;
use std::time::Duration;

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

impl PushMetricExporter for MetricExporter {
    /// Write Metrics to stdout
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Err(opentelemetry_sdk::error::OTelSdkError::AlreadyShutdown)
        } else {
            println!("Metrics");
            println!("Resource");
            if let Some(schema_url) = metrics.resource().schema_url() {
                println!("\tResource SchemaUrl: {schema_url:?}");
            }

            metrics.resource().iter().for_each(|(k, v)| {
                println!("\t ->  {k}={v:?}");
            });
            print_metrics(metrics.scope_metrics());
            Ok(())
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        // exporter holds no state, nothing to flush
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

fn print_metrics<'a>(metrics: impl Iterator<Item = &'a ScopeMetrics>) {
    for (i, metric) in metrics.enumerate() {
        println!("\tInstrumentation Scope #{i}");
        let scope = metric.scope();
        println!("\t\tName         : {}", scope.name());
        if let Some(version) = scope.version() {
            println!("\t\tVersion  : {version:?}");
        }
        if let Some(schema_url) = scope.schema_url() {
            println!("\t\tSchemaUrl: {schema_url:?}");
        }
        scope.attributes().enumerate().for_each(|(index, kv)| {
            if index == 0 {
                println!("\t\tScope Attributes:");
            }
            println!("\t\t\t ->  {}: {}", kv.key, kv.value);
        });

        metric.metrics().enumerate().for_each(|(i, metric)| {
            println!("Metric #{i}");
            println!("\t\tName         : {}", metric.name());
            println!("\t\tDescription  : {}", metric.description());
            println!("\t\tUnit         : {}", metric.unit());

            fn print_info<T>(data: &MetricData<T>)
            where
                T: Debug + Copy,
            {
                match data {
                    MetricData::Gauge(gauge) => {
                        println!("\t\tType         : Gauge");
                        print_gauge(gauge);
                    }
                    MetricData::Sum(sum) => {
                        println!("\t\tType         : Sum");
                        print_sum(sum);
                    }
                    MetricData::Histogram(hist) => {
                        println!("\t\tType         : Histogram");
                        print_histogram(hist);
                    }
                    MetricData::ExponentialHistogram(_) => {
                        println!("\t\tType         : Exponential Histogram");
                        // TODO: add support for ExponentialHistogram
                    }
                }
            }
            match metric.data() {
                AggregatedMetrics::F64(data) => print_info(data),
                AggregatedMetrics::U64(data) => print_info(data),
                AggregatedMetrics::I64(data) => print_info(data),
            }
        });
    }
}

fn print_sum<T: Debug + Copy>(sum: &Sum<T>) {
    println!("\t\tSum DataPoints");
    println!("\t\tMonotonic    : {}", sum.is_monotonic());
    if sum.temporality() == Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    let datetime: DateTime<Utc> = sum.start_time().into();
    println!(
        "\t\tStartTime    : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    let datetime: DateTime<Utc> = sum.time().into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    print_sum_data_points(sum.data_points());
}

fn print_gauge<T: Debug + Copy>(gauge: &Gauge<T>) {
    println!("\t\tGauge DataPoints");
    if let Some(start_time) = gauge.start_time() {
        let datetime: DateTime<Utc> = start_time.into();
        println!(
            "\t\tStartTime    : {}",
            datetime.format("%Y-%m-%d %H:%M:%S%.6f")
        );
    }
    let datetime: DateTime<Utc> = gauge.time().into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    print_gauge_data_points(gauge.data_points());
}

fn print_histogram<T: Debug + Copy>(histogram: &Histogram<T>) {
    if histogram.temporality() == Temporality::Cumulative {
        println!("\t\tTemporality  : Cumulative");
    } else {
        println!("\t\tTemporality  : Delta");
    }
    let datetime: DateTime<Utc> = histogram.start_time().into();
    println!(
        "\t\tStartTime    : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    let datetime: DateTime<Utc> = histogram.time().into();
    println!(
        "\t\tEndTime      : {}",
        datetime.format("%Y-%m-%d %H:%M:%S%.6f")
    );
    println!("\t\tHistogram DataPoints");
    print_hist_data_points(histogram.data_points());
}

fn print_sum_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a SumDataPoint<T>>,
) {
    for (i, data_point) in data_points.enumerate() {
        println!("\t\tDataPoint #{i}");
        println!("\t\t\tValue        : {:#?}", data_point.value());
        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

fn print_gauge_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a GaugeDataPoint<T>>,
) {
    for (i, data_point) in data_points.enumerate() {
        println!("\t\tDataPoint #{i}");
        println!("\t\t\tValue        : {:#?}", data_point.value());
        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }
    }
}

fn print_hist_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a HistogramDataPoint<T>>,
) {
    for (i, data_point) in data_points.enumerate() {
        println!("\t\tDataPoint #{i}");
        println!("\t\t\tCount        : {}", data_point.count());
        println!("\t\t\tSum          : {:?}", data_point.sum());
        if let Some(min) = &data_point.min() {
            println!("\t\t\tMin          : {min:?}");
        }

        if let Some(max) = &data_point.max() {
            println!("\t\t\tMax          : {max:?}");
        }

        println!("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            println!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str());
        }

        let mut lower_bound = f64::NEG_INFINITY;
        let bounds_iter = data_point.bounds();
        let mut bucket_counts_iter = data_point.bucket_counts();
        let mut header_printed = false;

        // Process all the regular buckets
        for upper_bound in bounds_iter {
            // Print header only once before the first item
            if !header_printed {
                println!("\t\t\tBuckets");
                header_printed = true;
            }

            // Get the count for this bucket, or 0 if not available
            let count = bucket_counts_iter.next().unwrap_or(0);
            println!("\t\t\t\t {lower_bound} to {upper_bound} : {count}");
            lower_bound = upper_bound;
        }

        // Handle the final +Infinity bucket if we processed any buckets
        if header_printed {
            let last_count = bucket_counts_iter.next().unwrap_or(0);
            println!("\t\t\t\t{lower_bound} to +Infinity : {last_count}");
        }
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
