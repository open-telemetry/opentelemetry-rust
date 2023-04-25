use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

const SLIDING_WINDOW_SIZE: u64 = 2; // In seconds

static STOP: AtomicBool = AtomicBool::new(false);

pub fn test_throughput<F>(func: F)
where
    F: Fn() + Sync,
{
    ctrlc::set_handler(move || {
        STOP.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Main loop
    let mut start_time = Instant::now();
    let mut end_time = Instant::now();
    let mut count: u64 = 0;
    let mut total_count: u64 = 0;
    loop {
        let elapsed = end_time.duration_since(start_time).as_secs();
        if elapsed >= SLIDING_WINDOW_SIZE {
            let throughput = count as f64 / elapsed as f64;
            println!("Throughput: {:.2} requests/sec", throughput);
            start_time = Instant::now();
            count = 0;
        }

        let num_threads = num_cpus::get();
        (0..num_threads).into_par_iter().for_each(|_| {
            func();
        });

        count += num_threads as u64;
        total_count += num_threads as u64;

        if STOP.load(Ordering::SeqCst) {
            break;
        }

        end_time = Instant::now();
    }

    println!("Total requests processed: {}", total_count);
}
