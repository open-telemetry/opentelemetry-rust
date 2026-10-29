//! In-Memory exporters for testing purpose.

/// Structs used for testing
#[cfg(all(feature = "testing", feature = "trace"))]
pub mod trace;

#[cfg(all(feature = "testing", feature = "metrics"))]
pub mod metrics;

#[cfg(all(
    test,
    feature = "testing",
    any(feature = "rt-tokio", feature = "rt-tokio-current-thread"),
    any(
        feature = "experimental_trace_batch_span_processor_with_async_runtime",
        feature = "experimental_logs_batch_log_processor_with_async_runtime",
        feature = "experimental_metrics_periodicreader_with_async_runtime"
    )
))]
pub(crate) fn run_in_subprocess() -> bool {
    use std::{
        process::Command,
        thread,
        time::{Duration, Instant},
    };

    let name = thread::current()
        .name()
        .expect("test thread should be named")
        .to_owned();
    if std::env::var("OTEL_SDK_TEST_CHILD").as_deref() == Ok(name.as_str()) {
        return false;
    }

    let mut child = Command::new(std::env::current_exe().expect("test executable should exist"))
        .args(["--exact", &name, "--nocapture", "--test-threads=1"])
        .env("OTEL_SDK_TEST_CHILD", &name)
        .stdin(std::process::Stdio::null())
        .spawn()
        .expect("test subprocess should start");

    // Synchronous flush and shutdown can block the runtime that would drive a timeout future.
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "test subprocess {name} failed: {status}");
                return true;
            }
            Ok(None) if start.elapsed() < Duration::from_secs(30) => {
                thread::sleep(Duration::from_millis(10));
            }
            result => {
                let kill_result = child.kill();
                let wait_result = child.wait();
                panic!(
                    "test subprocess {name} failed to finish within 30s: {result:?}; kill: {kill_result:?}; wait: {wait_result:?}"
                );
            }
        }
    }
}
