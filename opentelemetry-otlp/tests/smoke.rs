use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry_otlp::proto::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use std::{net::SocketAddr, sync::Mutex};
use tokio::sync::mpsc;

struct MockServer {
    tx: Mutex<mpsc::Sender<ExportTraceServiceRequest>>,
}

impl MockServer {
    pub fn new(tx: mpsc::Sender<ExportTraceServiceRequest>) -> Self {
        Self { tx: Mutex::new(tx) }
    }
}

#[tonic::async_trait]
impl TraceService for MockServer {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        println!("Sending request into channel...");
        self.tx
            .lock()
            .unwrap()
            .try_send(request.into_inner())
            .expect("Channel full");
        Ok(tonic::Response::new(ExportTraceServiceResponse {}))
    }
}

async fn setup() -> (SocketAddr, mpsc::Receiver<ExportTraceServiceRequest>) {
    let addr: SocketAddr = "[::1]:0".parse().unwrap();
    let mut listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().unwrap();
    let stream = async_stream::stream! {
        loop {
            let maybe_conn = listener.accept().await.map(|(c, addr)| {
                println!("Got new conn at {}", addr);
                c
            });
            yield maybe_conn;
        }
    };

    let (req_tx, req_rx) = mpsc::channel(10);
    let service = TraceServiceServer::new(MockServer::new(req_tx));
    tokio::task::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(service)
            .serve_with_incoming(stream)
            .await
            .expect("Server failed");
    });
    (addr, req_rx)
}

#[tokio::test(threaded_scheduler)]
async fn smoke_tracer() {
    println!("Starting server setup...");
    let (addr, mut req_rx) = setup().await;

    {
        println!("Installing tracer...");
        let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
            .with_endpoint(format!("http://{}", addr))
            .install()
            .expect("failed to install");

        println!("Sending span...");
        let span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event".into(), vec![]);
        span.end();
    }

    println!("Waiting for request...");
    let req = req_rx.recv().await.expect("missing export request");
    let first_span = req
        .resource_spans
        .get(0)
        .unwrap()
        .instrumentation_library_spans
        .get(0)
        .unwrap()
        .spans
        .get(0)
        .unwrap();
    assert_eq!("my-test-span", first_span.name);
    let first_event = first_span.events.get(0).unwrap();
    assert_eq!("my-test-event", first_event.name);
}
