# Using adaptive sampling with Jaeger
When services generate too many spans. We need to sample some spans to save cost and speed up the queries.

Adaptive sampling works in the Jaeger collector by observing the spans received from services and recalculating sampling probabilities for each service/endpoint combination to ensure that the volume is relatively constant.

## Setup
Start a jaeger collector and an opentelemetry collector locally using docker
```
docker-comopse run -d
```
It will allow you to 
- query sampling strategies from jaeger collect at port 5578.
- query sampling strategies from opentelemetry collector at port 5579

