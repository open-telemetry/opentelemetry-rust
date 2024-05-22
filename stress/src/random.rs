/*
    Stress test results:
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    1.25 B/sec
*/

use rand::{
    rngs::{self},
    Rng, SeedableRng,
};

mod throughput;

use std::cell::RefCell;

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
}

fn main() {
    throughput::test_throughput(test_random_generation);
}

fn test_random_generation() {
    let _i1 = CURRENT_RNG.with_borrow_mut(|rng| [rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10)]);
}
