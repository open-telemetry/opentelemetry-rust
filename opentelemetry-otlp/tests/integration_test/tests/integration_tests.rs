#![cfg(unix)]

use integration_test_runner::images::Collector;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use testcontainers::clients::Cli;
use testcontainers::core::Port;
use testcontainers::RunnableImage;

mod logs;
mod traces;

const COLLECTOR_CONTAINER_NAME: &str = "otel-collector";
const TEST_RESULT_DIR_IN_CONTAINER: &str = "testresults";
const EXPECTED_DIR: &str = "./expected";
const RESULT_FILE_PATH: &str = "./result.json";

struct TestSuite {
    expected_file_path: &'static str,
}

impl TestSuite {
    fn new(expected_file_path: &'static str) -> Self {
        Self { expected_file_path }
    }

    pub fn expected_file_path(&self) -> String {
        format!("{}/{}", EXPECTED_DIR, self.expected_file_path)
    }

    pub fn result_file_path_in_container(&self) -> String {
        format!("/{}/{}", TEST_RESULT_DIR_IN_CONTAINER, RESULT_FILE_PATH)
    }

    pub fn result_file_path(&self) -> String {
        format!("./{}", RESULT_FILE_PATH)
    }

    /// Create a empty file on localhost and copy it to container with proper permissions
    /// we have to create the file for the container otherwise we will encounter a permission denied error.
    /// see https://github.com/open-telemetry/opentelemetry-collector-contrib/issues/3159
    pub fn create_temporary_result_file(&self) -> File {
        let file = File::create(self.result_file_path()).unwrap();
        file.set_permissions(std::fs::Permissions::from_mode(0o666))
            .unwrap();
        file
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // skip when running unit test
async fn integration_tests() {
    trace_integration_tests().await;
    logs_integration_tests().await;
}

async fn trace_integration_tests() {
    let test_suites = [TestSuite::new("traces.json")];
    let mut collector_image = Collector::default();
    for test in test_suites.as_ref() {
        let _ = test.create_temporary_result_file();
        collector_image = collector_image.with_volume(
            test.result_file_path().as_str(),
            test.result_file_path_in_container().as_str(),
        );
    }

    let docker = Cli::default();
    let mut image =
        RunnableImage::from(collector_image).with_container_name(COLLECTOR_CONTAINER_NAME);

    for port in [
        4317, // gRPC port
        4318, // HTTP port
    ] {
        image = image.with_mapped_port(Port {
            local: port,
            internal: port,
        })
    }

    let collector_container = docker.run(image);

    tokio::time::sleep(Duration::from_secs(5)).await;
    traces::traces().await.unwrap();

    // wait for file to flush to disks
    // ideally we should use volume mount but otel collector file exporter doesn't handle permission too well
    // bind mount mitigate the issue by set up the permission correctly on host system
    tokio::time::sleep(Duration::from_secs(5)).await;
    traces::assert_traces_results(
        test_suites[0].result_file_path().as_str(),
        test_suites[0].expected_file_path().as_str(),
    );

    collector_container.stop();
}

async fn logs_integration_tests() {
    let test_suites = [TestSuite::new("logs.json")];

    let mut collector_image = Collector::default();
    for test in test_suites.as_ref() {
        let _ = test.create_temporary_result_file();
        collector_image = collector_image.with_volume(
            test.result_file_path().as_str(),
            test.result_file_path_in_container().as_str(),
        );
    }

    let docker = Cli::default();
    let mut image =
        RunnableImage::from(collector_image).with_container_name(COLLECTOR_CONTAINER_NAME);

    for port in [
        4317, // gRPC port
        4318, // HTTP port
    ] {
        image = image.with_mapped_port(Port {
            local: port,
            internal: port,
        })
    }

    let collector_container = docker.run(image);

    tokio::time::sleep(Duration::from_secs(5)).await;
    logs::logs().await.unwrap();

    // wait for file to flush to disks
    // ideally we should use volume mount but otel collector file exporter doesn't handle permission too well
    // bind mount mitigate the issue by set up the permission correctly on host system
    tokio::time::sleep(Duration::from_secs(5)).await;
    logs::assert_logs_results(
        test_suites[0].result_file_path().as_str(),
        test_suites[0].expected_file_path().as_str(),
    );

    collector_container.stop();
}
