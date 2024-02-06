#COMPOSE_FILE=./opentelemetry-jaeger/tests/docker-compose.yaml
#docker-compose -f $COMPOSE_FILE down -v &&
#docker-compose -f $COMPOSE_FILE up --build --abort-on-container-exit --exit-code-from opentelemetry-jaeger

cargo test ./opentelemetry-otlp/tests/integration_test/tests -- --ignored
