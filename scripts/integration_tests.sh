set -e

TEST_DIR="./opentelemetry-otlp/tests/integration_test/tests"

if [ -d "$TEST_DIR" ]; then
    cd "$TEST_DIR"

    # Run tests with the grpc-tonic feature
    echo
    echo ####
    echo Integration Tests: gRPC Tonic Client
    echo ####
    echo
    cargo test --no-default-features --features "tonic-client","internal-logs"

    # Run tests with the reqwest-client feature
    echo
    echo ####
    echo "Integration Tests: Reqwest Client"
    echo ####
    echo
    cargo test --no-default-features --features "reqwest-client","internal-logs"

    # Run tests with the reqwest-blocking-client feature
    echo
    echo ####
    echo Integration Tests: Reqwest Blocking Client
    echo ####
    echo
    cargo test --no-default-features --features "reqwest-blocking-client"

    # Run tests with the hyper-client feature
    echo
    echo ####
    echo "Integration Tests: Hyper Client (Disabled now)"
    echo ####
    echo
    cargo test --no-default-features --features "hyper-client","internal-logs" --test logs
else
    echo "Directory $TEST_DIR does not exist. Skipping tests."
    exit 1
fi
