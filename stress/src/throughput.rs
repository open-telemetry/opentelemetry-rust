use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

const SLIDING_WINDOW_SIZE: u64 = 2; // In seconds

static STOP: AtomicBool = AtomicBool::new(false);

pub fn test_throughput<F>(func: F)
where
    F: Fn() + Sync + Send + 'static,
{
    ctrlc::set_handler(move || {
        STOP.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    let mut start_time = Instant::now();
    let mut end_time = Instant::now();
    let mut total_count_old: u64 = 0;
    let total_count = Arc::new(AtomicU64::new(0));
    let total_count_clone = Arc::clone(&total_count);

    rayon::spawn( move || {
        (0..num_cpus::get()).into_par_iter().for_each(|_| loop {
            func();
            total_count_clone.fetch_add(1, Ordering::SeqCst);
            if STOP.load(Ordering::SeqCst) {
                break;
            }
        });
    });

    loop {
        let elapsed = end_time.duration_since(start_time).as_secs();
        if elapsed >= SLIDING_WINDOW_SIZE {
            let total_count_u64 = total_count.load(Ordering::Relaxed);
            let current_count = total_count_u64 - total_count_old;
            total_count_old = total_count_u64;
            let throughput = current_count as f64 / elapsed as f64;
            println!("Throughput: {:.2} requests/sec", throughput);
            start_time = Instant::now();
        }

        if STOP.load(Ordering::SeqCst) {
            break;
        }

        end_time = Instant::now();
    }
}
