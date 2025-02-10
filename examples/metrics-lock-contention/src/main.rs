use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use std::env::args;
use std::fmt::Debug;
use std::str::FromStr;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

const NUM_THREADS: usize = 10;

const RUN_TIME: Duration = Duration::from_secs(120);

fn main() {
    let mode: Mode = args().nth(1).unwrap_or("shared".to_string()).parse().unwrap();

    println!("Running with mode: {:?}, Duration: {}s, Threads: {}", mode, RUN_TIME.as_secs(), NUM_THREADS);

    let handles: Vec<JoinHandle<_>> = match mode {
        Mode::Shared =>  {
            let provider = create_meter_provider();
            (0..NUM_THREADS).map(move |_|start_work(provider.clone())).collect()
        }
        Mode::PerThread => {
            (0..NUM_THREADS).map(move |_|start_work(create_meter_provider())).collect()
        }
    };

    let sum = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .sum::<usize>();

    println!("Reported Metrics: {} millions", (sum / 1_000_000));
}

fn start_work(meter_provider: SdkMeterProvider) -> JoinHandle<usize> {
    let histogram = meter_provider.meter("dummy").f64_histogram("histogram").build();
    spawn(move || {
        let mut count = 0_usize;
        let now = Instant::now();

        loop {
            histogram.record(
                10.5,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                    KeyValue::new("mykey3", "myvalue3"),
                    KeyValue::new("mykey4", "myvalue4"),
                    KeyValue::new("mykey5", "myvalue5"),
                    KeyValue::new("mykey6", "myvalue6"),
                    KeyValue::new("mykey7", "myvalue7"),
                ],
            );

            count = count.checked_add(1).unwrap();

            if now.elapsed() > RUN_TIME {
                break;
            }
        }

        count
    })
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Shared,
    PerThread
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "shared" => Ok(Mode::Shared),
            "per-thread" => Ok(Mode::PerThread),
            _ => Err("invalid mode"),
        }
    }
}


fn create_meter_provider() -> SdkMeterProvider {
    SdkMeterProvider::builder()
        .with_reader(ManualReader::default())
        .build()
}
