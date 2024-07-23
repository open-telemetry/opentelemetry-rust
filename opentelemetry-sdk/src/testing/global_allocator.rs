// tests/global_allocator.rs
#[cfg(feature = "memory-profiling")]
mod global_allocator {
    use jemallocator::Jemalloc;

    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;
}
