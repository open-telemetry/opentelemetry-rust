# Run tests with the grpc-tonic feature
cd ./opentelemetry-otlp/tests/integration_test/tests && cargo test --no-default-features --features "tonic-client" -- --ignored

# Run tests with the reqwest-client feature
#cd ./opentelemetry-otlp/tests/integration_test/tests && cargo test --no-default-features --features "reqwest-client" -- --ignored

# Run tests with the reqwest-blocking-client feature
#cd ./opentelemetry-otlp/tests/integration_test/tests && cargo test --no-default-features --features "reqwest-blocking-client" -- --ignored

# Run tests with the hyper-client feature
#cd ./opentelemetry-otlp/tests/integration_test/tests && cargo test --no-default-features --features "hyper-client" -- --ignored
