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
cargo run --bin X
```

where `X` is the specific stress test you would like to run.

e.g.

```sh
cargo run --bin metrics
```

```text
Throughput: 70878.00 requests/sec
Throughput: 70272.00 requests/sec
Throughput: 70326.00 requests/sec
Throughput: 71724.00 requests/sec
Throughput: 74382.00 requests/sec
Throughput: 74856.00 requests/sec
Throughput: 75666.00 requests/sec
Total requests processed: 1035564
```
