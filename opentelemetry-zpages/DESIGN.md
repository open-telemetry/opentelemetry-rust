# Design proposal

## Problem statement
> zPages are an in-process alternative to external exporters. When included, they collect and aggregate tracing and metrics information in the background; this data is served on web pages when requested.

As noted in [Opentelemetry zPage spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/experimental/trace/zpages.md). zPage is a tool to help diagnose the application issues as well as the instrument issues without a external service.

There are several types of zPages defined in spec. Currently, we will only implement the tracez 

## Prior arts
Many language clients in OpenTelemetry alread implement at least part of the zPage service like [Cpp](https://github.com/open-telemetry/opentelemetry-cpp/blob/main/ext/src/zpages/README.md).

## Overall design
<details>
<summary>Diagram</summary>

```
                  ┌─────────────────────────┐              ┌────────────────────────┐
                  │                         │ ZPage Message│                        │
┌────────┐Regiser │ ZPage Span Processor    ├──────────────►  Span Aggregator       │
│ Span   ├────────►                         │              │                        │
└────────┘        └─────────────────────────┘              └───────────▲────────────┘
                                                                       │
                  ┌─────────────────────────┐                          │
                  │                         │                          │
                  │  Web Server             │                          │
                  │                         │                          │
                  │ ┌─────────────────┐     │  ZPage Query             │
                  │ │ Serilizer       │     ├──────────────────────────┘
                  │ │                 │     │
                  │ └─────────────────┘     │
                  │                         │
                  │                         │
                  └─────────────────────────┘
```
</details>

### ZPage Span Processor
This struct is needed mainly to integrate the existing tracing API. Most of its work will be delegated to `Span Aggregator`. This struct will implement `Span Processor` and `Tracez` trait.

### Span Aggregator
The Span aggregator will maintain a internal data storage to allow users track:
1. The number of current running spans.
2. The number of errored spans.
3. The number of spans in different latency buckets.
4. Current running spans examples.
5. Error spans examples.
6. Span examples with different run times distributed in 9 buckets.

The span aggregator should maintain a worker loop to handle the messages from the zpage span processor and web server. This worker loop should be non-blocking, so the zpage span processor will not block the span export at any point.


## Design ideas
### Span aggregator embedded into zpage span processor
One alternative choice other than using channels is to embed into the span aggregator. Then when span starts, span ends or there is an incoming http requests. We can lock the span aggregator to change the state. 

However, using this approach will block the `on_start` or `on_end` methods of zpage span processor if the span aggregator is working on serving a http request, which will further block the span processor chain to move forward when span ends.

This approach could have avoided the cloning when span starts. But unfortunately current span API doesn't allow us to get the span name without clone the `Span` into a `SpanData` object. Thus, the cloning cannot be avoided even if we embed the span aggregator into zpage span processor.

