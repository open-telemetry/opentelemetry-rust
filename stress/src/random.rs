/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~540 M/sec
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
    let _i1 = CURRENT_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        [
            rng.gen_range(0..10),
            rng.gen_range(0..10),
            rng.gen_range(0..10),
        ]
    });
}
