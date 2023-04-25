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

```text
Throughput: 4278494.00 requests/sec
Throughput: 4157096.50 requests/sec
Throughput: 4190185.50 requests/sec
Throughput: 4185362.50 requests/sec
Throughput: 4225514.00 requests/sec
Throughput: 4220563.00 requests/sec
Throughput: 4203673.50 requests/sec
Throughput: 4230679.50 requests/sec
```
