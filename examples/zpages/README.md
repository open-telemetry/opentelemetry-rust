# ZPages Example

In this example, we demonstrate how to use zpages to analysis spans. 

Run the following command to start the server on `localhost:3000`
```base
cargo run main
```

1. Then try to access `localhost:3000/running` endpoint. Each request sent to this endpoint will generate a trace whose latency is between 1 ms to 5 s. The latency for each trace will be printed in cmd.

2. Check `localhost:3000/api/tracez/aggregations` to see the count of running spans, error spans and spans within different latency.