# OpenTelemetry Stress Tests

## Why would you need stress test

* It helps you to understand performance.
* You can keep it running for days and nights to verify stability.
* You can use it to generate lots of load to your backend system.
* You can use it with other stress tools (e.g. a memory limiter) to verify how
  your code reacts to certain resource constraints.

## Usage

Open a console, run the following command from the current folder:

```sh
cargo run --release --bin X
```

where `X` is the specific stress test you would like to run.

e.g.

```sh
cargo run --release --bin metrics
```

Press (Ctrl + C) to quit the tests.

Example output:

```text
Throughput: 4813019.00 requests/sec
Throughput: 4805997.50 requests/sec
Throughput: 4796934.50 requests/sec
Throughput: 4817836.50 requests/sec
Throughput: 4812457.00 requests/sec
Throughput: 4773890.50 requests/sec
Throughput: 4760047.00 requests/sec
Throughput: 4851061.50 requests/sec
```
