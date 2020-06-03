//! Thrift HTTP transport
use std::fmt;
use std::sync::{Arc, Mutex};

/// Thrift channel over HTTP
#[derive(Clone)]
pub(crate) struct THttpChannel {
    endpoint: String,
    username: Option<String>,
    password: Option<String>,
    request: ureq::Request,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    write_buffer: Arc<Mutex<Vec<u8>>>,
}

impl fmt::Debug for THttpChannel {
    /// Debug info
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("THttpChannel")
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
        let endpoint: String = endpoint.into();
        let mut request = ureq::post(&endpoint);
        request.set("Content-Type", "application/vnd.apache.thrift.binary");
        // Some arbitrary default timeouts to avoid hanging forever
        request.timeout_connect(10000);
        request.timeout_read(10000);
        request.timeout_write(10000);

        if let (Some(username), Some(password)) = (username.as_ref(), password.as_ref()) {
            request.auth(&username, &password);
        }

        Ok(THttpChannel {
            request,
            endpoint,
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
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

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
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        write_buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    /// Send buffered data over HTTP and record response
    fn flush(&mut self) -> std::io::Result<()> {
        let mut write_buffer = self
            .write_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

        let mut req = self.request.build();
        let resp = req.send_bytes(&write_buffer.clone());
        if !resp.ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Expected success response, got {:?}", resp.status()),
            ));
        }

        write_buffer.clear();

        let has_content = resp
            .header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .map(|len| len > 0)
            .unwrap_or(false);
        if has_content {
            let mut response_reader = resp.into_reader();
            let mut read_buffer = self
                .read_buffer
                .lock()
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
            std::io::copy(&mut response_reader, &mut *read_buffer)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        }

        Ok(())
    }
}
