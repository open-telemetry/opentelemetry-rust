set -e
TEST_DIR="./opentelemetry-otlp/tests/integration_test/tests"

if [ -d "$TEST_DIR" ]; then
    cd "$TEST_DIR"
    # Run tests with the grpc-tonic feature
    cargo test --no-default-features --features "tonic-client" -- --ignored

    # Run tests with the reqwest-client feature
    cargo test --no-default-features --features "reqwest-client" -- --ignored

    # TODO - Uncomment the following lines once the reqwest-blocking-client feature is working.
    # cargo test --no-default-features --features "reqwest-blocking-client" -- --ignored

    # Run tests with the hyper-client feature
    cargo test --no-default-features --features "hyper-client" -- --ignored
else
    echo "Directory $TEST_DIR does not exist. Skipping tests."
    exit 1
fi
