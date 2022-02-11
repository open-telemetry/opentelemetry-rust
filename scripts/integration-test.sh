COMPOSE_FILE=./opentelemetry-jaeger/tests/docker-compose.yaml
docker-compose -f $COMPOSE_FILE down -v &&
docker-compose -f $COMPOSE_FILE up --build --exit-code-from opentelemetry-jaeger
