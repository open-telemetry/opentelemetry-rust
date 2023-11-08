# HTTP Example

This is a simple example using [hyper] that demonstrates tracing http request
from client to server, and from the server back to the client using the
[W3C Trace Context Response] header. The example shows key aspects of tracing
such as:

- Root Span (on Client)
- Child Span from a Remote Parent (on Server)
- SpanContext Propagation (from Client to Server)
- SpanContext Propagation (from Server to Client)
- Span Events
- Span Attributes

[hyper]: https://hyper.rs/
[W3C Trace Context Response]: https://w3c.github.io/trace-context/#traceresponse-header

## Usage

```shell
# Run server
$ cargo run --bin http-server

# In another tab, run client
$ cargo run --bin http-client

# The spans should be visible in stdout in the order that they were exported.
```
