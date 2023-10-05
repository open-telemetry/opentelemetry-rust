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
Number of threads: 4
Throughput: 4,714,600 iterations/sec
Throughput: 4,840,200 iterations/sec
Throughput: 3,905,200 iterations/sec
Throughput: 4,106,600 iterations/sec
Throughput: 5,075,400 iterations/sec
```
