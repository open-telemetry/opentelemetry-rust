use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use std::env::args;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

const NUM_THREADS: usize = 10;

const RUN_TIME: Duration = Duration::from_secs(120);

fn main() {
    let mode: Mode = args().nth(1).unwrap_or("shared".to_string()).parse().unwrap();
    let exit_signal = Arc::new(AtomicBool::new(false));

    println!("Running with mode: {:?}, Duration: {}s, Threads: {}", mode, RUN_TIME.as_secs(), NUM_THREADS);

    let handles: Vec<JoinHandle<_>> = match mode {
        Mode::Shared =>  {
            let provider = create_meter_provider();
            let signal = Arc::clone(&exit_signal);
            (0..NUM_THREADS).map(move |_|start_work(provider.clone(), signal.clone())).collect()
        }
        Mode::PerThread => {
            let signal = Arc::clone(&exit_signal);
            (0..NUM_THREADS).map(move |_|start_work(create_meter_provider(), signal.clone())).collect()
        }
    };

    _ = spawn(move || {
        std::thread::sleep(RUN_TIME);
        exit_signal.store(true, Ordering::Relaxed);
    });

    let sum = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .sum::<usize>();

    println!("Reported Metrics: {} millions", (sum / 1_000_000));
}

fn start_work(meter_provider: SdkMeterProvider, exit_signal: Arc<AtomicBool>) -> JoinHandle<usize> {
    let histogram = meter_provider.meter("dummy").f64_histogram("histogram").build();
    spawn(move || {
        let mut count = 0_usize;
        let now = Instant::now();

        loop {
            histogram.record(10.5, &[]);
            count = count.checked_add(1).unwrap();

            if exit_signal.load(Ordering::Relaxed) {
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
