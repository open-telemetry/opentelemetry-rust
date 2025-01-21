use num_format::{Locale, ToFormattedString};
use std::cell::UnsafeCell;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
#[cfg(feature = "stats")]
use sysinfo::{Pid, System};

const SLIDING_WINDOW_SIZE: u64 = 2; // In seconds

static STOP: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Default)]
struct WorkerStats {
    count: u64,
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

    let mut num_threads = num_cpus::get();
    let mut args_iter = env::args();

    if let Some(arg_str) = args_iter.nth(1) {
        let arg = arg_str.parse::<usize>();

        if arg.is_err() {
            eprintln!("Invalid command line argument '{}' as number of threads. Make sure the value is a positive integer.", arg_str);
            std::process::exit(1);
        }

        let arg_num = arg.unwrap();

        if arg_num > 0 {
            if arg_num > num_cpus::get() {
                println!(
                    "Specified {} threads which is larger than the number of logical cores ({})!",
                    arg_num, num_threads
                );
            }
            num_threads = arg_num;
        } else {
            eprintln!("Invalid command line argument {} as number of threads. Make sure the value is above 0 and less than or equal to number of available logical cores ({}).", arg_num, num_threads);
            std::process::exit(1);
        }
    }

    println!("Number of threads: {}\n", num_threads);
    let func_arc = Arc::new(func);
    let mut worker_stats_vec: Vec<WorkerStats> = Vec::new();

    for _ in 0..num_threads {
        worker_stats_vec.push(WorkerStats::default());
    }

    let shared_mutable_stats_slice = UnsafeSlice::new(&mut worker_stats_vec);

    thread::scope(|s| {
        s.spawn(|| {
            let mut last_collect_time = Instant::now();
            let mut total_count_old: u64 = 0;

            #[cfg(feature = "stats")]
            let pid = Pid::from(std::process::id() as usize);
            #[cfg(feature = "stats")]
            let mut system = System::new_all();

            loop {
                let current_time = Instant::now();
                let elapsed = current_time.duration_since(last_collect_time).as_secs();
                if elapsed >= SLIDING_WINDOW_SIZE {
                    let total_count_u64 = shared_mutable_stats_slice.sum();
                    last_collect_time = Instant::now();
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
                }

                if STOP.load(Ordering::SeqCst) {
                    break;
                }

                thread::sleep(Duration::from_millis(5000));
            }
        });

        for thread_index in 0..num_threads {
            let func_arc_clone = Arc::clone(&func_arc);
            s.spawn(move || loop {
                func_arc_clone();
                unsafe {
                    shared_mutable_stats_slice.increment(thread_index);
                }
                if STOP.load(Ordering::SeqCst) {
                    break;
                }
            });
        }
    });
}

#[derive(Copy, Clone)]
struct UnsafeSlice<'a> {
    slice: &'a [UnsafeCell<WorkerStats>],
}

unsafe impl Send for UnsafeSlice<'_> {}
unsafe impl Sync for UnsafeSlice<'_> {}

impl<'a> UnsafeSlice<'a> {
    fn new(slice: &'a mut [WorkerStats]) -> Self {
        let ptr = slice as *mut [WorkerStats] as *const [UnsafeCell<WorkerStats>];
        Self {
            slice: unsafe { &*ptr },
        }
    }

    // SAFETY: It's assumed that no two threads will write to the same index at the same time
    #[inline(always)]
    unsafe fn increment(&self, i: usize) {
        let value = self.slice[i].get();
        (*value).count += 1;
    }

    #[inline(always)]
    fn sum(&self) -> u64 {
        self.slice
            .iter()
            .map(|cell| unsafe { (*cell.get()).count })
            .sum()
    }
}
