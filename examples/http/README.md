# HTTP Example

This is a simple example using [hyper] that demonstrates tracing http request from client to server. The example
shows key aspects of tracing such as:

- Root Span (on Client)
- Child Span (on Client)
- Child Span from a Remote Parent (on Server)
- SpanContext Propagation (from Client to Server)
- Span Events
- Span Attributes

[hyper]: https://hyper.rs/

## Usage

```shell
# Run server
$ cargo run --bin http-server

# In another tab, run client
$ cargo run --bin http-client

# The spans should be visible in stdout in the order that they were exported.
```
