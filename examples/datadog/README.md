# Datadog Exporter Example

Sends spans to a datadog-agent collector.

## Usage 

First run version 7.22.0 or above of the datadog-agent locally as described [here](https://docs.datadoghq.com/agent/)

Then run the example to report spans:

```shell
$ cargo run
```

Traces should appear in the datadog APM dashboard
