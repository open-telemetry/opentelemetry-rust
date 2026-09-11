# OpenTelemetry Log Processor Implementation and Composition - Example

This example builds on top of `logs-basic`, showing how to implement and
compose `LogProcessor`s correctly.

`FilteringLogProcessor` drops records whose `event_id` attribute is `20`.
`EnrichmentLogProcessor` adds an attribute to records that pass the filter.
Both wrap and delegate to the next processor, preserving its lifecycle and
early `event_enabled` decisions. The filter is outermost so dropped records do
not incur enrichment work.

The stdout exporter therefore receives only the record with `event_id=50`.
The filter uses an attribute for illustration, but it could use any information
available to `LogProcessor::emit`.

## Usage

```shell
cargo run -p logs-advanced
```
