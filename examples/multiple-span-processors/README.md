# Work with multiple span processors

Opentelemetry supports export spans into multiple different destinations. One way to do so is to use multiple span processors. 

In this example, we demonstrate how to send spans to both Jaeger and Zipkin backend. 

To run this example. 

1. Start the Jaeger and Zipkin. Run `docker-compose up`

2. Use `cargo run` to run the example.

3. Check the output in Jaeger and Zipkin. The console should also output the SpanData in json format.

4. Use `docker-compose down -v` to tear down the Jaeger and Zipkin backend.


