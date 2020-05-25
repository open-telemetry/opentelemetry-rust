# Actix-web - Jaeger example with HTTP collector and batch exporter 

This example shows how to export spans from an actix-web application and ship them
 to the collector directly via HTTP.  
 It uses the batch exporter to avoid excessive network roundtrips to Jaeger.

## Usage

Launch the application:
```shell
# Run jaeger in background
$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest

# Start the actix-web server 
$ cargo run

# View spans (see the image below)
$ firefox http://localhost:16686/
```

Fire a request:
```bash
curl http://localhost:8088
```
