use futures_util::StreamExt;
use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, KeyPair};
use std::fs::{self, set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::{net::SocketAddr, sync::Mutex};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use tokio_stream::wrappers::TcpListenerStream;
#[cfg(feature = "gzip-tonic")]
use tonic::codec::CompressionEncoding;
use tonic::transport::{Identity, ServerTlsConfig};

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
        // assert we have required metadata key
        assert_eq!(
            request.metadata().get("x-header-key"),
            Some(&("header-value".parse().unwrap()))
        );
        self.tx
            .lock()
            .unwrap()
            .try_send(request.into_inner())
            .expect("Channel full");
        Ok(tonic::Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

async fn setup(
    tls_config: Option<ServerTlsConfig>,
) -> (SocketAddr, mpsc::Receiver<ExportTraceServiceRequest>) {
    let addr: SocketAddr = "[::1]:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().unwrap();
    let stream = TcpListenerStream::new(listener).map(|s| {
        if let Ok(ref s) = s {
            println!("Got new conn at {}", s.peer_addr().unwrap());
        }
        s
    });

    let (req_tx, req_rx) = mpsc::channel(10);
    #[cfg(feature = "gzip-tonic")]
    let service = TraceServiceServer::new(MockServer::new(req_tx))
        .accept_compressed(CompressionEncoding::Gzip);
    #[cfg(not(feature = "gzip-tonic"))]
    let service = TraceServiceServer::new(MockServer::new(req_tx));
    tokio::task::spawn(async move {
        let mut server = tonic::transport::Server::builder();
        if let Some(tls_config) = tls_config {
            server = server
                .tls_config(tls_config)
                .expect("failed to set tls config");
        }
        server
            .add_service(service)
            .serve_with_incoming(stream)
            .await
            .expect("Server failed")
    });
    (addr, req_rx)
}

#[tokio::test(flavor = "multi_thread")]
async fn smoke_tracer() {
    println!("Starting server setup...");
    let (addr, mut req_rx) = setup(None).await;

    {
        println!("Installing tracer provider...");
        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.insert("x-header-key", "header-value".parse().unwrap());
        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(
                #[cfg(feature = "gzip-tonic")]
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_compression(opentelemetry_otlp::Compression::Gzip)
                    .with_endpoint(format!("http://{addr}"))
                    .with_insecure()
                    .with_metadata(metadata)
                    .build()
                    .expect("gzip-tonic SpanExporter failed to build"),
                #[cfg(not(feature = "gzip-tonic"))]
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(format!("http://{}", addr))
                    .with_insecure()
                    .with_metadata(metadata)
                    .build()
                    .expect("NON gzip-tonic SpanExporter failed to build"),
            )
            .build();

        global::set_tracer_provider(tracer_provider.clone());

        let tracer = global::tracer("smoke");

        println!("Sending span...");
        let mut span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event", vec![]);
        span.end();

        tracer_provider
            .shutdown()
            .expect("tracer_provider should shutdown successfully");
    }

    println!("Waiting for request...");
    let req = req_rx.recv().await.expect("missing export request");
    let first_span = req
        .resource_spans
        .first()
        .unwrap()
        .scope_spans
        .first()
        .unwrap()
        .spans
        .first()
        .unwrap();
    assert_eq!("my-test-span", first_span.name);
    let first_event = first_span.events.first().unwrap();
    assert_eq!("my-test-event", first_event.name);
}

#[tokio::test(flavor = "multi_thread")]
async fn smoke_tls_tracer() {
    let (server_ca, server_cert, server_key) = generate_tls_certs();
    let (client_ca, client_cert, client_key) = generate_tls_certs();

    let server_ca_file = NamedTempFile::new().unwrap();
    let server_cert_file = NamedTempFile::new().unwrap();
    let server_key_file = NamedTempFile::new().unwrap();

    let client_ca_file = NamedTempFile::new().unwrap();
    let client_cert_file = NamedTempFile::new().unwrap();
    let client_key_file = NamedTempFile::new().unwrap();

    let files_and_contents = [
        (server_ca_file.path(), &server_ca),
        (server_cert_file.path(), &server_cert),
        (server_key_file.path(), &server_key),
        (client_ca_file.path(), &client_ca),
        (client_cert_file.path(), &client_cert),
        (client_key_file.path(), &client_key),
    ];

    for (file_path, content) in &files_and_contents {
        fs::write(file_path, content).unwrap();
    }

    let permissions = Permissions::from_mode(0o666);
    let files_to_set_permissions = [
        server_ca_file.path(),
        server_cert_file.path(),
        server_key_file.path(),
        client_ca_file.path(),
        client_cert_file.path(),
        client_key_file.path(),
    ];

    for file_path in &files_to_set_permissions {
        set_permissions(file_path, permissions.clone()).unwrap();
    }

    println!("Starting server setup...");
    let tls_config = ServerTlsConfig::new()
        .identity(Identity::from_pem(server_cert, server_key))
        .client_ca_root(tonic::transport::Certificate::from_pem(client_ca))
        .client_auth_optional(false);
    let (addr, mut req_rx) = setup(Some(tls_config)).await;

    {
        println!("Installing tracer provider...");
        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.insert("x-header-key", "header-value".parse().unwrap());
        let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(
                #[cfg(feature = "gzip-tonic")]
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_compression(opentelemetry_otlp::Compression::Gzip)
                    // Due a limitation in rustls, it's not possible to use the
                    // addr directely. It's not possible to use IP address. Domain
                    // name is required.
                    // https://github.com/hyperium/tonic/issues/279
                    .with_endpoint(format!("https://localhost:{}", addr.port()))
                    .with_certificate(server_ca_file.path().to_str().expect("Missing server CA"))
                    .with_client_certificate(
                        client_cert_file
                            .path()
                            .to_str()
                            .expect("Missing client certificate"),
                    )
                    .with_client_key(client_key_file.path().to_str().expect("Missing client key"))
                    .with_metadata(metadata)
                    .build()
                    .expect("gzip-tonic SpanExporter failed to build"),
                #[cfg(not(feature = "gzip-tonic"))]
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(format!("http://{}", addr))
                    .with_certificate(server_ca_file.path().to_str().expect("Missing server CA"))
                    .with_client_certificate(
                        client_cert_file
                            .path()
                            .to_str()
                            .expect("Missing client certificate"),
                    )
                    .with_client_key(client_key_file.path().to_str().expect("Missing client key"))
                    .with_metadata(metadata)
                    .build()
                    .expect("NON gzip-tonic SpanExporter failed to build"),
            )
            .build();

        global::set_tracer_provider(tracer_provider.clone());

        let tracer = global::tracer("smoke");

        println!("Sending span...");
        let mut span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event", vec![]);
        span.end();

        tracer_provider
            .shutdown()
            .expect("tracer_provider should shutdown successfully");
    }

    println!("Waiting for request...");
    let req = req_rx.recv().await.expect("missing export request");
    let first_span = req
        .resource_spans
        .first()
        .unwrap()
        .scope_spans
        .first()
        .unwrap()
        .spans
        .first()
        .unwrap();
    assert_eq!("my-test-span", first_span.name);
    let first_event = first_span.events.first().unwrap();
    assert_eq!("my-test-event", first_event.name);
}

fn generate_tls_certs() -> (String, String, String) {
    let ca_key = KeyPair::generate().unwrap();
    let mut params = CertificateParams::new(vec!["My Test CA".to_string()]).unwrap();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_cert = params.self_signed(&ca_key).unwrap();
    let ca_cert_pem = ca_cert.pem();

    let mut params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    params
        .distinguished_name
        .push(DnType::OrganizationName, "OpenTelemetry");
    params
        .distinguished_name
        .push(DnType::CommonName, "opentelemetry.io");

    let cert_key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&cert_key, &ca_cert, &ca_key).unwrap();
    let key = cert_key.serialize_pem();

    (ca_cert_pem, cert.pem(), key)
}
