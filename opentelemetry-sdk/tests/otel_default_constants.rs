//! Verifies the spec-defined `OTEL_*` environment variable names and their
//! `OTEL_*_DEFAULT` values are reachable from outside the crate.
//!
//! These live in an integration test rather than a unit test on purpose: an
//! integration test can only see the public API, so it also guards the
//! re-exports in `trace/mod.rs`, `logs/mod.rs`, and `metrics/mod.rs` against a
//! future refactor silently making them private again.
//!
//! See <https://github.com/open-telemetry/opentelemetry-rust/issues/3623>.

#[cfg(feature = "trace")]
#[test]
fn bsp_constants_are_public_and_match_spec_defaults() {
    use opentelemetry_sdk::trace::{
        OTEL_BSP_EXPORT_TIMEOUT, OTEL_BSP_EXPORT_TIMEOUT_DEFAULT, OTEL_BSP_MAX_CONCURRENT_EXPORTS,
        OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT, OTEL_BSP_MAX_EXPORT_BATCH_SIZE,
        OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT, OTEL_BSP_MAX_QUEUE_SIZE,
        OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT, OTEL_BSP_SCHEDULE_DELAY, OTEL_BSP_SCHEDULE_DELAY_DEFAULT,
    };
    use std::time::Duration;

    assert_eq!(OTEL_BSP_SCHEDULE_DELAY, "OTEL_BSP_SCHEDULE_DELAY");
    assert_eq!(
        OTEL_BSP_SCHEDULE_DELAY_DEFAULT,
        Duration::from_millis(5_000)
    );

    assert_eq!(OTEL_BSP_MAX_QUEUE_SIZE, "OTEL_BSP_MAX_QUEUE_SIZE");
    assert_eq!(OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT, 2_048);

    assert_eq!(
        OTEL_BSP_MAX_EXPORT_BATCH_SIZE,
        "OTEL_BSP_MAX_EXPORT_BATCH_SIZE"
    );
    assert_eq!(OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT, 512);

    assert_eq!(OTEL_BSP_EXPORT_TIMEOUT, "OTEL_BSP_EXPORT_TIMEOUT");
    assert_eq!(
        OTEL_BSP_EXPORT_TIMEOUT_DEFAULT,
        Duration::from_millis(30_000)
    );

    assert_eq!(
        OTEL_BSP_MAX_CONCURRENT_EXPORTS,
        "OTEL_BSP_MAX_CONCURRENT_EXPORTS"
    );
    assert_eq!(OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT, 1);
}

#[cfg(feature = "logs")]
#[test]
fn blrp_constants_are_public_and_match_spec_defaults() {
    use opentelemetry_sdk::logs::{
        OTEL_BLRP_MAX_EXPORT_BATCH_SIZE, OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
        OTEL_BLRP_MAX_QUEUE_SIZE, OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT, OTEL_BLRP_SCHEDULE_DELAY,
        OTEL_BLRP_SCHEDULE_DELAY_DEFAULT,
    };
    use std::time::Duration;

    assert_eq!(OTEL_BLRP_SCHEDULE_DELAY, "OTEL_BLRP_SCHEDULE_DELAY");
    assert_eq!(
        OTEL_BLRP_SCHEDULE_DELAY_DEFAULT,
        Duration::from_millis(1_000)
    );

    assert_eq!(OTEL_BLRP_MAX_QUEUE_SIZE, "OTEL_BLRP_MAX_QUEUE_SIZE");
    assert_eq!(OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT, 2_048);

    assert_eq!(
        OTEL_BLRP_MAX_EXPORT_BATCH_SIZE,
        "OTEL_BLRP_MAX_EXPORT_BATCH_SIZE"
    );
    assert_eq!(OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT, 512);
}

#[cfg(all(
    feature = "logs",
    feature = "experimental_logs_batch_log_processor_with_async_runtime"
))]
#[test]
fn blrp_export_timeout_constants_are_public_and_match_spec_defaults() {
    use opentelemetry_sdk::logs::{OTEL_BLRP_EXPORT_TIMEOUT, OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT};
    use std::time::Duration;

    assert_eq!(OTEL_BLRP_EXPORT_TIMEOUT, "OTEL_BLRP_EXPORT_TIMEOUT");
    assert_eq!(
        OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT,
        Duration::from_millis(30_000)
    );
}

#[cfg(feature = "metrics")]
#[test]
fn periodic_reader_constants_are_public_and_match_spec_defaults() {
    use opentelemetry_sdk::metrics::{
        OTEL_METRIC_EXPORT_INTERVAL, OTEL_METRIC_EXPORT_INTERVAL_DEFAULT,
    };
    use std::time::Duration;

    assert_eq!(OTEL_METRIC_EXPORT_INTERVAL, "OTEL_METRIC_EXPORT_INTERVAL");
    assert_eq!(OTEL_METRIC_EXPORT_INTERVAL_DEFAULT, Duration::from_secs(60));
}
