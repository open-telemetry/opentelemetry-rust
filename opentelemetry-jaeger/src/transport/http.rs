//! Thrift HTTP transport
use reqwest::header;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use thrift::{TransportError, TransportErrorKind};

/// Thrift channel over HTTP
#[derive(Clone)]
pub(crate) struct THttpChannel {
    client: reqwest::blocking::Client,
    endpoint: String,
    username: Option<String>,
    password: Option<String>,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    write_buffer: Arc<Mutex<Vec<u8>>>,
}

impl fmt::Debug for THttpChannel {
    /// Debug info
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("THttpChannel")
            .field("client", &self.client)
            .field("endpoint", &self.endpoint)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "************"))
            .field("read_buffer", &self.read_buffer)
            .field("write_buffer", &self.write_buffer)
            .finish()
    }
}

impl THttpChannel {
    /// Create a new `THttpChannel`
    pub(crate) fn new<T: Into<String>>(
        endpoint: T,
        username: Option<String>,
        password: Option<String>,
    ) -> thrift::Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/vnd.apache.thrift.binary"),
        );
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|err| {
                thrift::Error::Transport(TransportError::new(
                    TransportErrorKind::Unknown,
                    err.description(),
                ))
            })?;

        Ok(THttpChannel {
            client,
            endpoint: endpoint.into(),
            username,
            password,
            read_buffer: Default::default(),
            write_buffer: Default::default(),
        })
    }
}

impl thrift::transport::TIoChannel for THttpChannel {
    /// Split the channel into a readable half and a writable half.
    fn split(
        self,
    ) -> thrift::Result<(
        thrift::transport::ReadHalf<Self>,
        thrift::transport::WriteHalf<Self>,
    )>
    where
        Self: Sized,
    {
        Ok((
            thrift::transport::ReadHalf::new(self.clone()),
            thrift::transport::WriteHalf::new(self),
        ))
    }
}

impl std::io::Read for THttpChannel {
    /// Read from the channel's buffered responses.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut data = self
            .read_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;

        let amt = data.as_slice().read(buf)?;
        if amt > 0 {
            if amt == data.len() - 1 {
                data.clear();
            } else {
                let unread = data.split_off(amt);
                *data = unread;
            }
        }

        Ok(amt)
    }
}

impl std::io::Write for THttpChannel {
    /// Buffer data to send over HTTP
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut write_buffer = self
            .write_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;
        write_buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    /// Send buffered data over HTTP and record response
    fn flush(&mut self) -> std::io::Result<()> {
        let mut write_buffer = self
            .write_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;
        let mut req = self.client.post(&self.endpoint).body(write_buffer.clone());
        if let (Some(username), Some(password)) = (self.username.as_ref(), self.password.as_ref()) {
            req = req.basic_auth(username, Some(password));
        }
        let mut resp = req
            .send()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        if !resp.status().is_success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Expected success response, got {:?}", 1),
            ));
        }

        write_buffer.clear();

        if resp.content_length().filter(|len| *len > 0).is_some() {
            let mut read_buffer = self
                .read_buffer
                .lock()
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;

            resp.copy_to(&mut *read_buffer)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        }

        Ok(())
    }
}
