use num_format::{Locale, ToFormattedString};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
#[cfg(feature = "stats")]
use sysinfo::{Pid, System};

const SLIDING_WINDOW_SIZE: u64 = 2; // In seconds
const BATCH_SIZE: u64 = 1000;

static STOP: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Default)]
struct WorkerStats {
    count: AtomicU64,
    /// We use a padding for the struct to allow each thread to have exclusive access to each WorkerStat
    /// Otherwise, there would be some cpu contention with threads needing to take ownership of the cache lines
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
    let num_threads = num_cpus::get();
    println!("Number of threads: {}\n", num_threads);
    let mut handles = Vec::with_capacity(num_threads);
    let func_arc = Arc::new(func);
    let mut worker_stats_vec: Vec<WorkerStats> = Vec::new();

    for _ in 0..num_threads {
        worker_stats_vec.push(WorkerStats::default());
    }
    let worker_stats_shared = Arc::new(worker_stats_vec);
    let worker_stats_shared_monitor = Arc::clone(&worker_stats_shared);

    let handle_main_thread = thread::spawn(move || {
        let mut start_time = Instant::now();
        let mut end_time = start_time;
        let mut total_count_old: u64 = 0;

        #[cfg(feature = "stats")]
        let pid = Pid::from(std::process::id() as usize);
        #[cfg(feature = "stats")]
        let mut system = System::new_all();

        loop {
            let elapsed = end_time.duration_since(start_time).as_secs();
            if elapsed >= SLIDING_WINDOW_SIZE {
                let total_count_u64: u64 = worker_stats_shared_monitor
                    .iter()
                    .map(|worker_stat| worker_stat.count.load(Ordering::Relaxed))
                    .sum();
                let current_count = total_count_u64 - total_count_old;
                total_count_old = total_count_u64;
                let throughput = current_count / elapsed;
                println!(
                    "Throughput: {} iterations/sec",
                    throughput.to_formatted_string(&Locale::en)
                );

                #[cfg(feature = "stats")]
                {
                    system.refresh_all();
                    if let Some(process) = system.process(pid) {
                        println!(
                            "Memory usage: {:.2} MB",
                            process.memory() as f64 / (1024.0 * 1024.0)
                        );
                        println!("CPU usage: {}%", process.cpu_usage() / num_threads as f32);
                        println!(
                            "Virtual memory usage: {:.2} MB",
                            process.virtual_memory() as f64 / (1024.0 * 1024.0)
                        );
                    } else {
                        println!("Process not found");
                    }
                }

                println!("\n");
                start_time = Instant::now();
            }

            if STOP.load(Ordering::SeqCst) {
                break;
            }

            end_time = Instant::now();
            thread::sleep(Duration::from_millis(5000));
        }
    });

    handles.push(handle_main_thread);

    for thread_index in 0..num_threads - 1 {
        let worker_stats_shared = Arc::clone(&worker_stats_shared);
        let func_arc_clone = Arc::clone(&func_arc);
        let handle = thread::spawn(move || loop {
            for _ in 0..BATCH_SIZE {
                func_arc_clone();
            }
            worker_stats_shared[thread_index]
                .count
                .fetch_add(BATCH_SIZE, Ordering::Relaxed);
            if STOP.load(Ordering::SeqCst) {
                break;
            }
        });
        handles.push(handle)
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
