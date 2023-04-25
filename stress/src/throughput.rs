use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
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
    let counter = Arc::new(Mutex::new(vec![0; num_cpus::get()]));
    let mut total_count_old: u64 = 0;
    let counts = counter.clone();

    rayon::spawn(move || {
        (0..num_cpus::get()).into_par_iter().for_each(|i| {
            loop {
                func();
                let mut counts = counts.lock().unwrap();
                counts[i] += 1;
                if STOP.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
    });

    loop {

        let elapsed = end_time.duration_since(start_time).as_secs();
        if elapsed >= SLIDING_WINDOW_SIZE {
            let counts = counter.lock().unwrap();
            let total_count: usize = counts.iter().sum();
            let current_count = total_count as u64 - total_count_old;
            total_count_old = total_count as u64;
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
