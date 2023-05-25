use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

const SLIDING_WINDOW_SIZE: u64 = 2; // In seconds

static STOP: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Default)]
struct WorkerStats {
    count: AtomicU64,
    padding: [u64; 15],
}


pub fn test_throughput<F>(func: F)
where
    F: Fn() + Sync + Send + 'static,
{
    ctrlc::set_handler(move || {
        STOP.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    let num_threads = num_cpus::get_physical();
    println!("Number threads: {}", num_threads);
    let mut handles = Vec::with_capacity(num_threads);
    let func_arc = Arc::new(func);
    let mut worker_stats_vec: Vec<WorkerStats> = Vec::new();


    for _ in 0..num_threads {
        worker_stats_vec.push(WorkerStats::default());
    }
    let worker_stats_shared = Arc::new(worker_stats_vec);
    let worker_stats_shared_monitor = Arc::clone(&worker_stats_shared);

    // let total_count_clone = Arc::clone(&total_count);
    let handle1 = thread::spawn(move || {
        let mut start_time = Instant::now();
        let mut end_time = start_time;
        let mut total_count_old: u64 = 0;
        loop {
            let elapsed = end_time.duration_since(start_time).as_secs();
            if elapsed >= SLIDING_WINDOW_SIZE {
                // let total_count_u64 = total_count_clone.load(Ordering::Relaxed);
                let total_count_u64: u64 = worker_stats_shared_monitor.iter().map(|worker_stat| worker_stat.count.load(Ordering::Relaxed)).sum();
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
            thread::sleep(Duration::from_millis(5000));
        }
    });

    handles.push(handle1);

    for thread_index in 0..num_threads - 1 {
        let worker_stats_shared = Arc::clone(&worker_stats_shared);
        let func_arc_clone = Arc::clone(&func_arc);
        let handle = thread::spawn(move || {
            loop {
                for _ in 0..1000 {
                    func_arc_clone();
                }
                worker_stats_shared[thread_index].count.fetch_add(1000, Ordering::Relaxed);
                // total_count_clone.fetch_add(1000, Ordering::Relaxed);
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
