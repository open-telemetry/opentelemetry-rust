use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

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
    let mut total_count_old: usize = 0;

    let num_threads = num_cpus::get();
    const BATCH_SIZE: usize = 10000;
    let index_array: Arc<Vec<AtomicUsize>> = Arc::new((0..num_threads).map(|_| AtomicUsize::new(0)).collect());
    let thread_references: Vec<_> = (0..num_threads)
    .map(|_| Arc::clone(&index_array))
    .collect();

    rayon::spawn(move || {
        thread_references.into_par_iter().enumerate().for_each(|(thread_index, thread_array)| {
            let mut local_count = 0;
            loop {
                for _ in 0..BATCH_SIZE {
                    func();
                    local_count += 1;
                }
                thread_array[thread_index].fetch_add(local_count, Ordering::Relaxed);
                local_count = 0;
                if STOP.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
    });

    loop {
        let elapsed = end_time.duration_since(start_time).as_secs();
        if elapsed >= SLIDING_WINDOW_SIZE {
            let total_count: usize = index_array.iter().map(|atomic| atomic.load(Ordering::Relaxed)).sum();
            let current_count = total_count - total_count_old;
            total_count_old = total_count;
            let throughput = current_count as u64 / elapsed;
            println!("Throughput: {:.2} requests/sec", throughput);
            start_time = Instant::now();
        }

        if STOP.load(Ordering::SeqCst) {
            break;
        }

        end_time = Instant::now();
        thread::sleep(Duration::from_millis(1));
    }
}
