use rand::{rngs::SmallRng, Rng, SeedableRng};

mod throughput;

fn main() {
    throughput::test_throughput(test_random_generation);
}

fn test_random_generation() {
    let mut rng = SmallRng::from_entropy();
    let _index_first_attribute = rng.gen_range(0..10);
    let _index_second_attribute = rng.gen_range(0..10);
    let _index_third_attribute = rng.gen_range(0..10);
}
