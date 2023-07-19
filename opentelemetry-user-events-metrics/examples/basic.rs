//! run with `$ cargo run --example basic --all-features
use opentelemetry_api::{
    metrics::{MeterProvider as _, Unit},
    KeyValue,
};
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime, Resource,
};
use opentelemetry_user_events_metrics::MetricsExporter;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliOptions {
    #[structopt(short = "a", long = "account", value_name = "ACCOUNT", help = "Sets metrics account")]
    metrics_account: Option<String>,

    #[structopt(short = "n", long = "namespace", value_name = "NAMESPACE", help = "Sets the metrics namespace")]
    metrics_namespace: Option<String>,
}

fn init_metrics(exporter: MetricsExporter) -> MeterProvider {
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    MeterProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "metric-demo",
        )]))
        .with_reader(reader)
        .build()
}

#[tokio::main]
#[allow(unused_must_use)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = opentelemetry_user_events_metrics::MetricsExporter::new();
    let meter_provider = init_metrics(exporter);

    let meter = meter_provider.versioned_meter(
        "user-event-test",
        Some("test-version"),
        Some("test_url"),
        Some(vec![KeyValue::new("key", "value")]),
    );
    let c = meter
        .f64_counter("counter_test")
        .with_description("test_decription")
        .with_unit(Unit::new("test_unit"))
        .init();

    let mut attributes: Vec<KeyValue> = Vec::new();
    attributes.push(KeyValue::new("mykey1", "myvalue1"));
    attributes.push(KeyValue::new("mykey2", "myvalue2"));
    let options = CliOptions::from_args();

    // Access the values of the arguments
    if let Some(metrics_account) = options.metrics_account {
        println!("Account: {}", metrics_account);
        attributes.push(KeyValue::new("_microsoft_metrics_account", metrics_account));
    }

    if let Some(metrics_namespace) = options.metrics_namespace {
        println!("Namespace: {}", metrics_namespace);
        attributes.push(KeyValue::new("_microsoft_metrics_namespace", metrics_namespace));
    }

    c.add(1.0, &attributes);

    meter_provider.shutdown()?;

    Ok(())
}
