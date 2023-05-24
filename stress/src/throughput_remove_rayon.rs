use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
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
    let num_threads = num_cpus::get();
    let mut handles = Vec::with_capacity(num_threads);
    let total_count = Arc::new(AtomicU64::new(0));
    let func_arc = Arc::new(func);

    let total_count_clone = Arc::clone(&total_count);
    let handle1 = thread::spawn(move || {
        let mut start_time = Instant::now();
        let mut end_time = start_time;
        let mut total_count_old: u64 = 0;
        loop {
            let elapsed = end_time.duration_since(start_time).as_secs();
            if elapsed >= SLIDING_WINDOW_SIZE {
                let total_count_u64 = total_count_clone.load(Ordering::Relaxed);
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
    });

    handles.push(handle1);

    for _ in 0..num_threads {
        let total_count_clone = Arc::clone(&total_count);
        let func_arc_clone = Arc::clone(&func_arc);
        let handle = thread::spawn(move || {
            loop {
                // let func = Arc::clone(&func_arc_clone);
                func_arc_clone();
                total_count_clone.fetch_add(1, Ordering::Relaxed);
                if STOP.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        handles.push(handle)
    }

    for handle in handles {
        handle.join().unwrap();
    }

}
