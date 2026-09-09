use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

const MAX_REQUEST_SIZE: usize = 1024 * 1024;

#[derive(Debug)]
pub struct ScriptedHttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
}

impl ScriptedHttpResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }
}

#[derive(Clone, Debug)]
pub struct CapturedHttpRequest {
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub received_at: Instant,
}

#[derive(Debug)]
pub struct FakeOtlpHttpEndpoint {
    address: SocketAddr,
    requests: Arc<Mutex<Vec<CapturedHttpRequest>>>,
    errors: Arc<Mutex<Vec<String>>>,
    server: JoinHandle<()>,
}

impl FakeOtlpHttpEndpoint {
    pub async fn start(responses: Vec<ScriptedHttpResponse>) -> io::Result<Self> {
        assert!(
            !responses.is_empty(),
            "fake OTLP endpoint requires at least one response"
        );

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured_requests = Arc::clone(&requests);
        let errors = Arc::new(Mutex::new(Vec::new()));
        let server_errors = Arc::clone(&errors);
        let mut responses = VecDeque::from(responses);

        let server = tokio::spawn(async move {
            loop {
                let (mut stream, _) = match listener.accept().await {
                    Ok(connection) => connection,
                    Err(error) => {
                        server_errors.lock().unwrap().push(error.to_string());
                        break;
                    }
                };
                let request = match read_request(&mut stream).await {
                    Ok(request) => request,
                    Err(error) => {
                        server_errors.lock().unwrap().push(error.to_string());
                        continue;
                    }
                };
                captured_requests.lock().unwrap().push(request);

                let response = responses
                    .pop_front()
                    // Fall back to 500 if more requests arrive than scripted responses;
                    // this makes unexpected extra requests fail visibly in assertions.
                    .unwrap_or_else(|| ScriptedHttpResponse::new(500));
                if let Err(error) = write_response(&mut stream, response).await {
                    server_errors.lock().unwrap().push(error.to_string());
                }
            }
        });

        Ok(Self {
            address,
            requests,
            errors,
            server,
        })
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    pub fn requests(&self) -> Vec<CapturedHttpRequest> {
        let errors = self.errors.lock().unwrap();
        assert!(
            errors.is_empty(),
            "fake OTLP endpoint encountered errors: {errors:?}"
        );
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for FakeOtlpHttpEndpoint {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn read_request(stream: &mut TcpStream) -> io::Result<CapturedHttpRequest> {
    let mut bytes = Vec::new();
    let header_end = loop {
        if bytes.len() >= MAX_REQUEST_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request headers exceeded size limit",
            ));
        }

        let mut buffer = [0; 4096];
        let read = stream.read(&mut buffer).await?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed before request headers",
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);

        if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
    };

    let header_text = std::str::from_utf8(&bytes[..header_end]).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("request headers were not valid UTF-8: {error}"),
        )
    })?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().unwrap_or_default().to_owned();
    let mut headers = HashMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("malformed request header: {line}"),
            ));
        };
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }

    if headers.contains_key("transfer-encoding") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "fake OTLP endpoint does not support transfer-encoding",
        ));
    }
    let content_length = headers
        .get("content-length")
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "OTLP request did not include content-length",
            )
        })?
        .parse::<usize>()
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid content-length: {error}"),
            )
        })?;
    if header_end + content_length > MAX_REQUEST_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "request body exceeded size limit",
        ));
    }

    while bytes.len() < header_end + content_length {
        let mut buffer = [0; 4096];
        let read = stream.read(&mut buffer).await?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed before request body",
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
    }

    Ok(CapturedHttpRequest {
        request_line,
        headers,
        body: bytes[header_end..header_end + content_length].to_vec(),
        received_at: Instant::now(),
    })
}

async fn write_response(stream: &mut TcpStream, response: ScriptedHttpResponse) -> io::Result<()> {
    let reason = match response.status {
        200 => "OK",
        400 => "Bad Request",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Test Response",
    };
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: 0\r\nConnection: close\r\n",
        response.status, reason
    );
    for (name, value) in response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");

    stream.write_all(head.as_bytes()).await?;
    stream.shutdown().await
}
