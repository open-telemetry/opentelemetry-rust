# Jaeger remote sampler

When services generate too many spans. We need to sample some spans to save cost and speed up the queries.

Adaptive sampling works in the Jaeger collector by observing the spans received from services and recalculating sampling
probabilities for each service/endpoint combination to ensure that the volume is relatively constant.

For a full list of configurations. See SDK docs of [JaegerRemoteSamplerBuilder](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.JaegerRemoteSamplerBuilder.html).

## Setup

Start a jaeger collector and an opentelemetry collector locally using docker

```
docker-compose run -d
```

It will allow you to

- query sampling strategies from jaeger collect at port 5578. `http://localhost:5778/sampling?service=foo`
- query sampling strategies from opentelemetry collector at port 5579. `http://localhost:5779/sampling?service=foo`

## Run the example

After start the jaeger remote sampling server successfully. We can run

`cargo run`

command to start the example, you should only see one span is printed out. 

Looking at the example, you will notice we use `AlwaysOff` as our default sampler. It means before the SDK get the sampling strategy from remote server, no span will be sampled. 

Once the SDK fetched the remote strategy, we will start a probability sampler internally. In this case, we set the probability to 1.0 for all spans. This is defined by

```
"service": "foo",
"type": "probabilistic",
"param": 1,
```

Feel free to tune the `param` and see if the probability of sampling changes. 

## Strategies

The sampling strategies is defined in `srategies.json` files. It defines two set of strategies.

The first strategy is returned for `foo` service. The second strategy is catch all default strategy for all other
services.
