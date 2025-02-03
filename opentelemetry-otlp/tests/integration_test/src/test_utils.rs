//! Supporting infrastructure for OTLP integration tests.
//!
//! This module provides the pieces needed to work with an actual opentelemetry-collector
//! instance, which is started in Docker and has its output plumbed back into the host filesystem.
//! This lets us write tests that push data over OTLP (HTTP or gRPC) to the collector, and then read
//! that data back from the filesystem to ensure everything worked out as expected.
//!
//! To use this module, all you need to do is call `start_collector_container()` from each
//! of your tests, and use a single `#[dtor]` at the end of your test file to call
//! `stop_collector_container`. Note that as cargo integration tests run a process-per-test-file,
//! each test will get its own fresh instance of the container.
//!
//! Only a single test suite can run at once, as each container has statically mapped ports, but
//! this works nicely with the way cargo executes the suite.
//!
//! To skip integration tests with cargo, you can run `cargo test --mod`, which will run unit tests
//! only.
//!
#![cfg(unix)]

use anyhow::Result;
use opentelemetry::{otel_debug, otel_info};
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex, Once, OnceLock};
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::core::{ContainerPort, Mount};
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

// Static references for container management
static COLLECTOR_ARC: OnceLock<Mutex<Option<Arc<ContainerAsync<GenericImage>>>>> = OnceLock::new();

pub static METRICS_FILE: &str = "./actual/metrics.json";
pub static LOGS_FILE: &str = "./actual/logs.json";
pub static TRACES_FILE: &str = "./actual/traces.json";

static INIT_TRACING: Once = Once::new();

fn init_tracing() {
    INIT_TRACING.call_once(|| {
        // Info and above for all, debug for opentelemetry
        let filter_fmt =
            EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_filter(filter_fmt);

        // Initialize the tracing subscriber with the OpenTelemetry layer and the
        // Fmt layer.
        tracing_subscriber::registry().with(fmt_layer).init();
        otel_info!(name: "tracing::fmt initializing completed! SDK internal logs will be printed to stdout.");
    });
}

pub async fn start_collector_container() -> Result<()> {
    init_tracing();

    let mut arc_guard = COLLECTOR_ARC
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap();

    // If the container isn't running, start it.
    if arc_guard.is_none() {
        // Make sure all our test data is mounted
        upsert_empty_file(METRICS_FILE);
        upsert_empty_file(TRACES_FILE);
        upsert_empty_file(LOGS_FILE);

        // Start a new container
        let container_instance = GenericImage::new("otel/opentelemetry-collector", "latest")
            .with_wait_for(WaitFor::http(
                HttpWaitStrategy::new("/")
                    .with_expected_status_code(404u16)
                    .with_port(ContainerPort::Tcp(4318)),
            ))
            .with_mapped_port(4317, ContainerPort::Tcp(4317))
            .with_mapped_port(4318, ContainerPort::Tcp(4318))
            .with_mount(Mount::bind_mount(
                fs::canonicalize("./otel-collector-config.yaml")?.to_string_lossy(),
                "/etc/otelcol/config.yaml",
            ))
            .with_mount(Mount::bind_mount(
                fs::canonicalize("./actual/logs.json")?.to_string_lossy(),
                "/testresults/logs.json",
            ))
            .with_mount(Mount::bind_mount(
                fs::canonicalize("./actual/metrics.json")?.to_string_lossy(),
                "/testresults/metrics.json",
            ))
            .with_mount(Mount::bind_mount(
                fs::canonicalize("./actual/traces.json")?.to_string_lossy(),
                "/testresults/traces.json",
            ))
            .start()
            .await?;

        let container = Arc::new(container_instance);
        otel_info!(
            name: "Container started",
            ports = format!("{:?}", container.ports().await));

        // Give the container a second to stabilize
        //tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Store the container in COLLECTOR_ARC
        *arc_guard = Some(Arc::clone(&container));
    } else {
        otel_info!(name: "OTel Collector already running");
    }

    Ok(())
}

///
/// Creates an empty file with permissions that make it usable both within docker
/// and on the host.
///
fn upsert_empty_file(path: &str) -> File {
    let file = File::create(path).unwrap();
    file.set_permissions(std::fs::Permissions::from_mode(0o666))
        .unwrap();
    file
}

/// Cleans up file specificed as argument by truncating its content.
///
/// This function is meant to cleanup the generated json file before a test starts,
/// preventing entries from previous tests from interfering with the current test's results.
pub fn cleanup_file(file_path: &str) {
    let _ = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path); // ignore result, as file may not exist
}

///
/// Shuts down our collector container. This should be run as part of each test
/// suite shutting down!
///
pub fn stop_collector_container() {
    // This is a bit heinous. We don't have an async runtime left when
    // we hit this call, so we can't use the async methods on the testcontainers
    // interface to shutdown.
    // We _need_ to do this here, because otherwise we have no "all the tests in the module
    // were complete" hook.
    //
    // https://github.com/testcontainers/testcontainers-rs/issues/707
    otel_debug!(name: "stop_collector_container");

    if let Some(mutex_option_arc) = COLLECTOR_ARC.get() {
        let guard = mutex_option_arc.lock().unwrap();
        if let Some(container_arc) = &*guard {
            std::process::Command::new("docker")
                .args(["container", "rm", "-f", container_arc.id()])
                .output()
                .expect("failed to stop testcontainer");
        }
    }
}
