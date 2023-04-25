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
Throughput: 412998.00 requests/sec
Throughput: 404226.00 requests/sec
Throughput: 417000.00 requests/sec
Throughput: 403440.00 requests/sec
Throughput: 395148.00 requests/sec
Throughput: 404172.00 requests/sec
Throughput: 395412.00 requests/sec
Throughput: 400740.00 requests/sec
Throughput: 390132.00 requests/sec
Total requests processed: 7418976
```
