# OpenTelemetry Rust Metrics

<details>
<summary>Table of Contents</summary>

* [Best Practices](#best-practices)
* [Metrics API](#metrics-api)
  * [Meter](#meter)
  * [Instruments](#instruments)
  * [Reporting measurements - use array slices for
    attributes](#reporting-measurements---use-array-slices-for-attributes)
  * [Reporting measurements via synchronous
    instruments](#reporting-measurements-via-synchronous-instruments)
  * [Reporting measurements via asynchronous
    instruments](#reporting-measurements-via-asynchronous-instruments)
* [MeterProvider Management](#meterprovider-management)
* [Memory Management](#memory-management)
  * [Example](#example)
  * [Pre-Aggregation](#pre-aggregation)
    * [Pre-Aggregation Benefits](#pre-aggregation-benefits)
  * [Cardinality Limits](#cardinality-limits)
    * [Cardinality Limits - Implications](#cardinality-limits---implications)
    * [Cardinality Limits - Example](#cardinality-limits---example)
  * [Memory Preallocation](#memory-preallocation)
* [Metrics Correlation](#metrics-correlation)
* [Modelling Metric Attributes](#modelling-metric-attributes)

</details>

## Best Practices

// TODO: Add link to the examples, once they are modified to show best
practices.

## Metrics API

### Meter

:stop_sign: You should avoid creating duplicate
[`Meter`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Meter.html)
instances with the same name. `Meter` is fairly expensive and meant to be reused
throughout the application. For most applications, a `Meter` should be obtained
from `global` and saved for re-use.

The fully qualified module name might be a good option for the Meter name.
Optionally, one may create a meter with version, schema_url, and additional
meter-level attributes as well. Both are shown in below examples.

```rust
use opentelemetry::global;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;

let scope = InstrumentationScope::builder("my_company.my_product.my_library")
        .with_version("0.17")
        .with_schema_url("https://opentelemetry.io/schema/1.2.0")
        .with_attributes([KeyValue::new("key", "value")])
        .build();

// creating Meter with InstrumentationScope, comprising of
// name, version, schema and attributes.
let meter = global::meter_with_scope(scope);

// creating Meter with just name
let meter = global::meter("my_company.my_product.my_library");
```

### Instruments

:heavy_check_mark: You should understand and pick the right instrument type.

> [!NOTE] Picking the right instrument type for your use case is crucial to
> ensure the correct semantics and performance. Check the [Instrument
  Selection](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/supplementary-guidelines.md#instrument-selection)
  section from the supplementary guidelines for more information.

| OpenTelemetry Specification | OTel Rust Instrument Type |
| --------------------------- | -------------------- |
| [Asynchronous Counter](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#asynchronous-counter) | [`ObservableCounter`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.ObservableCounter.html) |
| [Asynchronous Gauge](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#asynchronous-gauge) | [`ObservableGauge`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.ObservableGauge.html) |
| [Asynchronous UpDownCounter](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#asynchronous-updowncounter) | [`ObservableUpDownCounter`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.ObservableUpDownCounter.html) |
| [Counter](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#counter) | [`Counter`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Counter.html) |
| [Gauge](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#gauge) | [`Gauge`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Gauge.html) |
| [Histogram](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#histogram) | [`Histogram`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Histogram.html) |
| [UpDownCounter](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#updowncounter) | [`UpDownCounter`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.UpDownCounter.html) |

:stop_sign: You should avoid creating duplicate instruments (e.g., `Counter`)
with the same name. Instruments are fairly expensive and meant to be reused
throughout the application. For most applications, an instrument should be
created once and saved for re-use. Instruments can also be cloned to create
multiple handles to the same instrument, but cloning should not occur on the hot
path. Instead, the cloned instance should be stored and reused.

:stop_sign: Do NOT use invalid instrument names.

> [!NOTE] OpenTelemetry will not collect metrics from instruments that are using
> invalid names. Refer to the [OpenTelemetry
  Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#instrument-name-syntax)
  for the valid syntax.

:stop_sign: You should avoid changing the order of attributes while reporting
measurements.

> [!WARNING] The last line of code has bad performance since the attributes are
> not following the same order as before:

```rust
let counter = meter.u64_counter("fruits_sold").build();
counter.add(2, &[KeyValue::new("color", "red"), KeyValue::new("name", "apple")]);
counter.add(3, &[KeyValue::new("color", "green"), KeyValue::new("name", "lime")]);
counter.add(5, &[KeyValue::new("color", "yellow"), KeyValue::new("name", "lemon")]);
counter.add(8, &[KeyValue::new("name", "lemon"), KeyValue::new("color", "yellow")]); // bad performance
```

:heavy_check_mark: If feasible, provide the attributes sorted by `Key`s in
ascending order to minimize memory usage within the Metrics SDK.

```rust
let counter = meter.u64_counter("fruits_sold").build();
counter.add(2, &[KeyValue::new("color", "red"), KeyValue::new("name", "apple")]);
```

### Reporting measurements - use array slices for attributes

:heavy_check_mark:  When reporting measurements, use array slices for attributes
rather than creating vectors. Arrays are more efficient as they avoid
unnecessary heap allocations on the measurement path. This is true for both
synchronous and observable instruments.

```rust
// Good practice: Using an array slice directly
counter.add(2, &[KeyValue::new("color", "red"), KeyValue::new("name", "apple")]);

// Usi
let _observable_counter = meter
        .u64_observable_counter("request_count")
        .with_callback(|observer| {
            // Good practice: Using an array slice directly
            observer.observe(
                100,
                &[KeyValue::new("endpoint", "/api")]
            )
        })
        .build();

// Avoid this: Creating a Vec is unnecessary, and it allocates on the heap
// counter.add(2, &vec![KeyValue::new("color", "red"), KeyValue::new("name", "apple")]);
```

### Reporting measurements via synchronous instruments

:heavy_check_mark: Use synchronous Counter when you need to increment counts at
specific points in your code:

```rust
// Example: Using Counter when incrementing at specific code points
use opentelemetry::KeyValue;

fn process_item(counter: &opentelemetry::metrics::Counter<u64>, item_type: &str) {
    // Process item...
    
    // Increment the counter with the item type as an attribute
    counter.add(1, &[KeyValue::new("type", item_type)]);
}
```

### Reporting measurements via asynchronous instruments

Asynchronous instruments like `ObservableCounter` are ideal for reporting
metrics that are already being tracked or stored elsewhere in your application.
These instruments allow you to observe and report the current state of such
metric without manually managing the aggregation yourself.

:heavy_check_mark: Use `ObservableCounter` when you already have a variable
tracking a count:

```rust
// Example: Using ObservableCounter when you already have a variable tracking counts
use opentelemetry::KeyValue;
use std::sync::atomic::{AtomicU64, Ordering};

// An existing variable in your application
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

// In your application code, you update this directly
fn handle_request() {
    // Process request...
    REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
}

// When setting up metrics, register an observable counter that reads from your variable
fn setup_metrics(meter: &opentelemetry::metrics::Meter) {
    let _observable_counter = meter
        .u64_observable_counter("request_count")
        .with_description("Number of requests processed")
        .with_unit("requests")
        .with_callback(|observer| {
            // Read the current value from your existing counter
            observer.observe(
                REQUEST_COUNT.load(Ordering::SeqCst), 
                &[KeyValue::new("endpoint", "/api")]
            )
        })
        .build();
}
```

> [!NOTE] The callbacks in the Observable instruments are invoked by the SDK
during each export cycle.

## MeterProvider Management

Most use-cases require you to create ONLY one instance of MeterProvider. You
should NOT create multiple instances of MeterProvider unless you have some
unusual requirement of having different export strategies within the same
application. Using multiple instances of MeterProvider requires users to
exercise caution..

:heavy_check_mark: Properly manage the lifecycle of `MeterProvider` instances if
you create them. Follow these guidelines:

* **Cloning**: A `MeterProvider` is a handle to an underlying provider. Cloning
  it creates a new handle pointing to the same provider. Clone the
  `MeterProvider` when necessary, but re-use the cloned instead of repeatedly
  cloning.

* **Set as Global Provider**: Use `opentelemetry::global::set_meter_provider` to
  set a clone of the `MeterProvider` as the global provider. This ensures
  consistent usage across the application, allowing applications and libraries
  to obtain `Meter` from the global instance.

* **Shutdown**: Explicitly call `shutdown` on the `MeterProvider` at the end of
  your application to ensure all metrics are properly flushed and exported.

> [!NOTE] If you did not use opentelemetry::global::set_meter_provider to set a
> clone of the MeterProvider as the global provider, then you should be aware
> that dropping the last instance of MeterProvider implicitly call shutdown on
> the provider.

:heavy_check_mark: Always call `shutdown` on the `MeterProvider` at the end of
your application to ensure proper cleanup.

## Memory Management

In OpenTelemetry,
[measurements](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#measurement)
are reported via the metrics API. The SDK
[aggregates](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#aggregation)
metrics using certain algorithms and memory management strategies to achieve
good performance and efficiency. Here are the rules which OpenTelemetry Rust
follows while implementing the metrics aggregation logic:

1. [**Pre-Aggregation**](#pre-aggregation): aggregation occurs within the SDK.
2. [**Cardinality Limits**](#cardinality-limits): the aggregation logic respects
   [cardinality
   limits](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#cardinality-limits),
   so the SDK does not use indefinite amount of memory when there is cardinality
   explosion.
3. [**Memory Preallocation**](#memory-preallocation): SDK tries to pre-allocate
   the memory it needs at each instrument creation time.

### Example

Let us take the following example of OTel Rust metrics being used to track the
number of fruits sold:

* During the time range (T0, T1]:
  * value = 1, name = `apple`, color = `red`
  * value = 2, name = `lemon`, color = `yellow`
* During the time range (T1, T2]:
  * no fruit has been sold
* During the time range (T2, T3]:
  * value = 5, name = `apple`, color = `red`
  * value = 2, name = `apple`, color = `green`
  * value = 4, name = `lemon`, color = `yellow`
  * value = 2, name = `lemon`, color = `yellow`
  * value = 1, name = `lemon`, color = `yellow`
  * value = 3, name = `lemon`, color = `yellow`

### Example - Cumulative Aggregation Temporality

If we aggregate and export the metrics using [Cumulative Aggregation
Temporality](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/data-model.md#temporality):

* (T0, T1]
  * attributes: {name = `apple`, color = `red`}, count: `1`
  * attributes: {name = `lemon`, color = `yellow`}, count: `2`
* (T0, T2]
  * attributes: {name = `apple`, color = `red`}, count: `1`
  * attributes: {name = `lemon`, color = `yellow`}, count: `2`
* (T0, T3]
  * attributes: {name = `apple`, color = `red`}, count: `6`
  * attributes: {name = `apple`, color = `green`}, count: `2`
  * attributes: {name = `lemon`, color = `yellow`}, count: `12`

Note that the start time is not advanced, and the exported values are the
cumulative total of what happened since the beginning.

### Example - Delta Aggregation Temporality

If we aggregate and export the metrics using [Delta Aggregation
Temporality](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/data-model.md#temporality):

* (T0, T1]
  * attributes: {name = `apple`, color = `red`}, count: `1`
  * attributes: {name = `lemon`, color = `yellow`}, count: `2`
* (T1, T2]
  * nothing since we do not have any measurement received
* (T2, T3]
  * attributes: {name = `apple`, color = `red`}, count: `5`
  * attributes: {name = `apple`, color = `green`}, count: `2`
  * attributes: {name = `lemon`, color = `yellow`}, count: `10`

Note that the start time is advanced after each export, and only the delta since
last export is exported, allowing SDK to "forget" previous state.

### Pre-Aggregation

Using the [fruit example](#example), there are six measurements reported during
the time range `(T2, T3]`. Instead of exporting each individual measurement
event, the SDK aggregates them and exports only the summarized results. This
process, illustrated in the following diagram, is known as pre-aggregation:

```mermaid
graph LR

subgraph SDK
    Instrument --> | Measurements | Pre-Aggregation[Pre-Aggregation]
end

subgraph Collector
    Aggregation
end

Pre-Aggregation --> | Metrics | Aggregation
```

In addition to the in-process aggregation performed by the OpenTelemetry Rust
Metrics SDK, further aggregations can be carried out by the Collector and/or the
metrics backend.

### Pre-Aggregation Benefits

Pre-aggregation offers several advantages:

1. **Reduced Data Volume**: Summarizes measurements before export, minimizing
   network overhead and improving efficiency.
2. **Predictable Resource Usage**: Ensures consistent resource consumption by
   applying [cardinality limits](#cardinality-limits) and [memory
   preallocation](#memory-preallocation) during SDK initialization. In other
   words, metrics memory/network usage remains capped, regardless of the volume
   of measurements being made.This ensures that resource utilization remains
   stable despite fluctuations in traffic volume.
3. **Improved Performance**: Reduces serialization costs as we work with
   aggregated data and not the numerous individual measurements. It also reduces
   computational load on downstream systems, enabling them to focus on analysis
   and storage.

> [!NOTE] There is no ability to opt out of pre-aggregation.

### Cardinality Limits

The number of unique combinations of attributes for a given metric is referred
to as the cardinality of that metric. Taking the [fruit example](#example), if
we know that we can only have apple/lemon as the name, red/yellow/green as the
color, then we can say the cardinality is 6 (i.e 2 * 3). No matter how many
fruits we sell, we can always use the following table to summarize the total
number of fruits based on the name and color.

| Name  | Color  | Count |
| ----- | ------ | ----- |
| apple | red    | 6     |
| apple | yellow | 0     |
| apple | green  | 2     |
| lemon | red    | 0     |
| lemon | yellow | 12    |
| lemon | green  | 0     |

In other words, we know how much memory and network are needed to collect and
transmit these metrics, regardless of the traffic pattern or volume.

In real world applications, the cardinality can be extremely high. Imagine if we
have a long running service and we collect metrics with 7 attributes and each
attribute can have 30 different values. We might eventually end up having to
remember the complete set of 30⁷ - or 21.87 billion combinations! This
cardinality explosion is a well-known challenge in the metrics space. For
example, it can cause surprisingly high costs in the observability system, or
even be leveraged by bad actors to launch a denial-of-service attack.

[Cardinality
limit](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#cardinality-limits)
is a throttling mechanism which allows the metrics collection system to have a
predictable and reliable behavior when there is a cardinality explosion, be it
due to a malicious attack or developer making mistakes while writing code.

OpenTelemetry has a default cardinality limit of `2000` per metric. This limit
can be configured at the individual metric level using the [View
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#view)
leveraging the
[`cardinality_limit`](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.Stream.html#structfield.cardinality_limit)
setting.

It's important to understand that this cardinality limit applies only at the
OpenTelemetry SDK level, not to the ultimate cardinality of the metric as seen
by the backend system. For example, while a single process might be limited to
2000 attribute combinations per metric, the actual backend metrics system might
see much higher cardinality due to:

1. Resource attributes (such as `service.instance.id`, `host.name`, etc.) that
   can be added to each metric
2. Multiple process instances running the same application across your
   infrastructure
3. The possibility of reporting different key-value pair combinations in each
   export interval, as the cardinality limit only applies to the number of
   unique attribute combinations tracked during a single export interval. (This
   is only applicable to Delta temporality)

Therefore, the actual cardinality in your metrics backend can be orders of
magnitude higher than what any single OTel SDK process handles in an export
cycle.

#### Cardinality Limits - Implications

Cardinality limits are enforced for each export interval, meaning the metrics
aggregation system only allows up to the configured cardinality limit of unique
attribute combinations per metric. Understanding how this works in practice is
important:

* **Cardinality Capping**: When the limit is reached within an export interval,
  any new attribute combination is not individually tracked but instead folded
  into a single aggregation with the attribute `{"otel.metric.overflow": true}`.
  This preserves the overall accuracy of aggregates (such as Sum, Count, etc.)
  even though information about specific attribute combinations is lost. Every
  measurement is accounted for - either with its original attributes or within
  the overflow bucket.

* **Temporality Effects**: The impact of cardinality limits differs based on the
  temporality mode:
  
  * **Delta Temporality**: The SDK "forgets" the state after each
    collection/export cycle. This means in each new interval, the SDK can track
    up to the cardinality limit of completely different attribute combinations.
    Over time, your metrics backend might see far more than the configured limit
    of unique combinations from a single process.
  
  * **Cumulative Temporality**: Since the SDK maintains state across export
    intervals, once the cardinality limit is reached, new attribute combinations
    will continue to be folded into the overflow bucket. The total number of
    distinct attribute combinations exported cannot exceed the cardinality limit
    for the lifetime of that metric instrument.

* **Impact on Monitoring**: While cardinality limits protect your system from
  unbounded resource consumption, they do mean that high-cardinality dimensions
  may not be fully represented in your metrics. Since cardinality capping can
  cause metrics to be folded into the overflow bucket, it becomes impossible to
  predict which specific attribute combinations were affected across multiple
  collection cycles or different service instances.

  This unpredictability creates several important considerations when querying
  metrics in any backend system:
  
  * **Total Accuracy**: OpenTelemetry Metrics always ensures the total
    aggregation (sum of metric values across all dimensions) remains accurate,
    even when overflow occurs.
  
  * **Attribute-Based Query Limitations**: Any metric query based on specific
    attributes could be misleading, as it's possible those dimensions were
    folded into the overflow bucket due to cardinality capping.
  
  * **All Attributes Affected**: When overflow occurs, it's not just
    high-cardinality attributes that are affected. The entire attribute
    combination is replaced with the `{"otel.metric.overflow": true}` attribute,
    meaning queries for any attribute in that combination will miss data points.

#### Cardinality Limits - Example

Extending our fruit sales tracking example, imagine we set a
cardinality limit of 3 and we're tracking sales with attributes for `name`,
`color`, and `store_location`:

During a busy sales period at time (T3, T4], we record:
  
1. 10 red apples sold at Downtown store
2. 5 yellow lemons sold at Uptown store
3. 8 green apples sold at Downtown store
4. 3 red apples sold at Midtown store (at this point, the cardinality limit is
    hit, and attributes are replaced with overflow attribute.)

The exported metrics would be:
  
* attributes: {name = `apple`, color = `red`, store_location = `Downtown`},
    count: `10`
* attributes: {name = `lemon`, color = `yellow`, store_location = `Uptown`},
    count: `5`
* attributes: {name = `apple`, color = `green`, store_location = `Downtown`},
    count: `8`
* attributes: {`otel.metric.overflow` = `true`}, count: `3`
  
  If we later query "How many red apples were sold?" the answer would be 10, not
  13, because the Midtown sales were folded into the overflow bucket. Similarly,
  queries about "How many items were sold in Midtown?" would return 0, not 3.
  However, the total count across all dimensions (i.e How many total fruits were
  sold in (T3, T4] would correctly give 26) would be accurate.

  This limitation applies regardless of whether the attribute in question is
  naturally high-cardinality. Even low-cardinality attributes like "color"
  become unreliable for querying if they were part of attribute combinations
  that triggered overflow.

  OpenTelemetry's cardinality capping is only applied to attributes provided
  when reporting measurements via the [Metrics API](#metrics-api). In other
  words, attributes used to create `Meter` or `Resource` attributes are not
  subject to this cap.

  :heavy_check_mark: Use `Meter` attributes or `Resource` attributes as
  appropriate, see [modelling attributes](#modelling-metric-attributes) for
  details. In the above example, if a process only sells fruits from a
  particular store, then store_location attribute should be modelled as a
  Resource - this is not only efficient, but reduced the cardinality capping
  possibility as well.

// TODO: Document how to pick cardinality limit.

### Memory Preallocation

OpenTelemetry Rust SDK aims to avoid memory allocation on the hot code path.
When this is combined with [proper use of Metrics API](#metrics-api), heap
allocation can be avoided on the hot code path.

## Metrics Correlation

Including `TraceId` and `SpanId` as attributes in metrics might seem like an
intuitive way to achieve correlation with traces or logs. However, this approach
is ineffective and can make metrics practically unusable. Moreover, it can
quickly lead to cardinality issues, resulting in metrics being capped.

A better alternative is to use a concept in OpenTelemetry called
[exemplars](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#exemplar).
Exemplars provide a mechanism to correlate metrics with traces by sampling
specific measurements and attaching trace context to them.

Currently, exemplars are not yet implemented in the OpenTelemetry Rust SDK.

## Modelling Metric Attributes

When metrics are being collected, they normally get stored in a [time series
database](https://en.wikipedia.org/wiki/Time_series_database). From storage and
consumption perspective, metrics can be multi-dimensional. Taking the [fruit
example](#example), there are two dimensions - "name" and "color". For basic
scenarios, all the dimensions can be reported during the [Metrics
API](#metrics-api) invocation, however, for less trivial scenarios, the
dimensions can come from different sources:

* [Measurements](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#measurement)
  reported via the [Metrics API](#metrics-api).
* Additional attributes provided at meter creation time via
  [`meter_with_scope`](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/trait.MeterProvider.html#tymethod.meter_with_scope).
* [Resources](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md)
  configured at the `MeterProvider` level.
* Additional attributes provided by the collector. For example, [jobs and
  instances](https://prometheus.io/docs/concepts/jobs_instances/) in Prometheus.

Here is the rule of thumb when modeling the dimensions:

* If the dimension is static throughout the process lifetime (e.g. the name of
  the machine, data center):
  * If the dimension applies to all metrics, model it as Resource, or even
    better, let the collector add these dimensions if feasible (e.g. a collector
    running in the same data center should know the name of the data center,
    rather than relying on / trusting each service instance to report the data
    center name).
  * If the dimension applies to a subset of metrics (e.g. the version of a
    client library), model it as meter level attributes.
* If the dimension value is dynamic, report it via the [Metrics
  API](#metrics-api). Be aware that [cardinality limits](#cardinality-limits)
  are applied to these attributes.

## Common issues that lead to missing metrics

// TODO
