//! Benchmark different definition styles for the `LogExporter` trait.
//!
//! Run this benchmark with the following command:
//! ```sh
//! cargo bench --bench log_exporter
//! ```
//!
//! Example benchmark results:
//! - date: 2024-09-24
//! - OS: Linux 6.10.10 x86_64 (Arch Linux)
//! - CPU: AMD Ryzen 9 7950X
//! - RAM: 64 GiB
//! ```text
//! LogExporterAsyncTrait   time:   [60.918 ns 61.146 ns 61.404 ns]
//! LogExporterAsyncNative  time:   [51.846 ns 51.905 ns 51.979 ns]
//! LogExporterSync         time:   [50.197 ns 50.332 ns 50.472 ns]
//! ```

use std::sync::Mutex;
use std::time::SystemTime;

use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

use opentelemetry::logs::{LogRecord as _, LogResult, Logger, LoggerProvider as _, Severity};

use opentelemetry::InstrumentationLibrary;
use opentelemetry_sdk::export::logs::LogBatch;
use opentelemetry_sdk::logs::LogProcessor;
use opentelemetry_sdk::logs::LogRecord;
use opentelemetry_sdk::logs::LoggerProvider;
use pprof::criterion::{Output, PProfProfiler};
use std::fmt::Debug;
use std::hint::black_box;

mod style_async_trait {
    use super::*;

    /// Async style using the `async_trait` crate
    #[async_trait]
    pub trait LogExporterAsyncTrait: Send + Sync + Debug {
        async fn export(&mut self, batch: LogBatch<'_>);
    }

    #[derive(Debug)]
    pub struct NoOpExporterAsyncTrait;

    #[async_trait]
    impl LogExporterAsyncTrait for NoOpExporterAsyncTrait {
        async fn export(&mut self, batch: LogBatch<'_>) {
            // As of writing this benchmark, the `rust-version` is `1.65`, but
            // this benchmark relies on native async trait support from `1.75`
            // so we may as well use the `black_box` hint from `1.66`. Benchmark
            // are expected to be executed with a recent Rust toolchain anyway.
            #[allow(clippy::incompatible_msrv)]
            black_box(batch);
        }
    }

    #[derive(Debug)]
    pub struct ExportingProcessorAsyncTrait {
        exporter: Mutex<NoOpExporterAsyncTrait>,
    }

    impl ExportingProcessorAsyncTrait {
        pub fn new(exporter: NoOpExporterAsyncTrait) -> Self {
            Self {
                exporter: Mutex::new(exporter),
            }
        }
    }

    impl LogProcessor for ExportingProcessorAsyncTrait {
        fn emit(&self, record: &mut LogRecord, library: &InstrumentationLibrary) {
            let mut exporter = self.exporter.lock().expect("lock error");
            let logs = [(record as &LogRecord, library)];
            futures_executor::block_on(exporter.export(LogBatch::new(&logs)));
        }

        fn force_flush(&self) -> LogResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            Ok(())
        }
    }
}

mod style_async_native {
    use super::*;

    /// Async style using Rustc support for async traits
    pub trait LogExporterAsyncNative: Send + Sync + Debug {
        fn export(&mut self, batch: LogBatch<'_>) -> impl std::future::Future<Output = ()> + Send;
    }

    #[derive(Debug)]
    pub struct NoOpExporterAsyncNative;

    impl LogExporterAsyncNative for NoOpExporterAsyncNative {
        #[inline]
        async fn export(&mut self, batch: LogBatch<'_>) {
            // As of writing this benchmark, the `rust-version` is `1.65`, but
            // this benchmark relies on native async trait support from `1.75`
            // so we may as well use the `black_box` hint from `1.66`. Benchmark
            // are expected to be executed with a recent Rust toolchain anyway.
            #[allow(clippy::incompatible_msrv)]
            black_box(batch);
        }
    }

    #[derive(Debug)]
    pub struct ExportingProcessorAsyncNative {
        exporter: Mutex<NoOpExporterAsyncNative>,
    }

    impl ExportingProcessorAsyncNative {
        pub fn new(exporter: NoOpExporterAsyncNative) -> Self {
            Self {
                exporter: Mutex::new(exporter),
            }
        }
    }

    impl LogProcessor for ExportingProcessorAsyncNative {
        #[inline]
        fn emit(&self, record: &mut LogRecord, library: &InstrumentationLibrary) {
            let mut exporter = self.exporter.lock().expect("lock error");
            let logs = [(record as &LogRecord, library)];
            futures_executor::block_on(exporter.export(LogBatch::new(&logs)));
        }

        fn force_flush(&self) -> LogResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            Ok(())
        }
    }
}

mod style_sync {
    use super::*;

    /// Sync style
    pub trait LogExporterSync: Send + Sync + Debug {
        fn export(&mut self, batch: LogBatch<'_>);
    }

    #[derive(Debug)]
    pub struct NoOpExporterSync;

    impl LogExporterSync for NoOpExporterSync {
        fn export(&mut self, batch: LogBatch<'_>) {
            // As of writing this benchmark, the `rust-version` is `1.65`, but
            // this benchmark relies on native async trait support from `1.75`
            // so we may as well use the `black_box` hint from `1.66`. Benchmark
            // are expected to be executed with a recent Rust toolchain anyway.
            #[allow(clippy::incompatible_msrv)]
            black_box(batch);
        }
    }

    #[derive(Debug)]
    pub struct ExportingProcessorSync {
        exporter: Mutex<NoOpExporterSync>,
    }

    impl ExportingProcessorSync {
        pub fn new(exporter: NoOpExporterSync) -> Self {
            Self {
                exporter: Mutex::new(exporter),
            }
        }
    }

    impl LogProcessor for ExportingProcessorSync {
        fn emit(&self, record: &mut LogRecord, library: &InstrumentationLibrary) {
            let mut exporter = self.exporter.lock().expect("lock error");
            let logs = [(record as &LogRecord, library)];
            exporter.export(LogBatch::new(&logs));
        }

        fn force_flush(&self) -> LogResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> LogResult<()> {
            Ok(())
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_async_trait(c);
    bench_async_native(c);
    bench_sync(c);
}

fn bench_async_trait(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(style_async_trait::ExportingProcessorAsyncTrait::new(
            style_async_trait::NoOpExporterAsyncTrait,
        ))
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("LogExporterAsyncTrait", |b| {
        b.iter(|| workload(&logger));
    });
}

fn bench_async_native(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(style_async_native::ExportingProcessorAsyncNative::new(
            style_async_native::NoOpExporterAsyncNative,
        ))
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("LogExporterAsyncNative", |b| {
        b.iter(|| workload(&logger));
    });
}

fn bench_sync(c: &mut Criterion) {
    let provider = LoggerProvider::builder()
        .with_log_processor(style_sync::ExportingProcessorSync::new(
            style_sync::NoOpExporterSync,
        ))
        .build();
    let logger = provider.logger("benchmark");

    c.bench_function("LogExporterSync", |b| {
        b.iter(|| workload(&logger));
    });
}

/// Dummy workload to exercise a logger
fn workload<TyLogger>(logger: &TyLogger)
where
    TyLogger: Logger,
{
    let mut log_record = logger.create_log_record();
    let now = SystemTime::now();
    log_record.set_observed_timestamp(now);
    log_record.set_target("my-target".to_string());
    log_record.set_event_name("CheckoutFailed");
    log_record.set_severity_number(Severity::Warn);
    log_record.set_severity_text("WARN");
    log_record.add_attribute("book_id", "12345");
    log_record.add_attribute("book_title", "Rust Programming Adventures");
    log_record.add_attribute("message", "Unable to process checkout.");

    logger.emit(log_record);
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}
criterion_main!(benches);
