use opentelemetry_api::metrics::Unit;
use opentelemetry_api::{
    metrics::MeterProvider as _,
    Context, KeyValue,
};
use opentelemetry_sdk::metrics::{PeriodicReader, MeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_stdout;
use std::error::Error;

fn init_meter_provider() -> MeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    MeterProvider::builder().with_reader(reader).with_resource(Resource::new(vec![
        KeyValue::new("service.name", "metrics-basic-example"),
    ])).build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    
    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = meter_provider.meter("mylibraryname");

    // Create a gauge instrument from the above Meter and register a callback that reports the measurement.
    let gauge = meter
        .f64_observable_gauge("my_gauge")
        .with_description("A gauge set to 1.0")
        .with_unit(Unit::new("myunit"))
        .init();

    // Register a callback that reports the measurement.
    meter.register_callback(&[gauge.as_any()], move |observer| {
        observer.observe_f64(&gauge, 1.0, [KeyValue::new("mykey1", "myvalue1"), KeyValue::new("mykey2", "myvalue2")].as_ref())
    })?;

    // Create a Histogram Instrument.
    let histogram = meter.f64_histogram("my_histogram").with_description("My histogram example description").init();

    // Record measurements using the histogram instrument.
    histogram.record(&Context::current(), 10.5, [KeyValue::new("mykey1", "myvalue1"), KeyValue::new("mykey2", "myvalue2")].as_ref());

    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").init();

    // Record measurements using the Counter instrument.
    counter.add(&Context::current(), 10, [KeyValue::new("mykey1", "myvalue1"), KeyValue::new("mykey2", "myvalue2")].as_ref());

    // Metrics are exported by default every 30 seconds,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
