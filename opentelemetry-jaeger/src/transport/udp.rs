//! Thrift UDP transport
use std::error::Error;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::{Arc, Mutex};
use thrift::transport::{ReadHalf, WriteHalf};

/// Max size of datagram in bytes
const UDP_PACKET_MAX_LENGTH: usize = 65000;

#[derive(Clone, Debug)]
pub(crate) struct TUdpChannel {
    conn: Arc<UdpSocket>,
    max_packet_size: usize,
    write_buffer: Arc<Mutex<Vec<u8>>>,
}

impl thrift::transport::TIoChannel for TUdpChannel {
    fn split(self) -> thrift::Result<(ReadHalf<Self>, WriteHalf<Self>)>
    where
        Self: Sized,
    {
        Ok((ReadHalf::new(self.clone()), WriteHalf::new(self)))
    }
}

impl TUdpChannel {
    pub(crate) fn new<T: ToSocketAddrs>(
        host_port: T,
        max_packet_size: Option<usize>,
    ) -> thrift::Result<Self> {
        let max_packet_size = max_packet_size.unwrap_or(UDP_PACKET_MAX_LENGTH);

        let conn = UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;

        Ok(TUdpChannel {
            conn: Arc::new(conn),
            max_packet_size,
            write_buffer: Default::default(),
        })
    }
}

impl std::io::Write for TUdpChannel {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut write_buffer = self
            .write_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;
        if write_buffer.len() + buf.len() > self.max_packet_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "input ({} bytes) is larger than max packet size ({} bytes)",
                    write_buffer.len() + buf.len(),
                    self.max_packet_size
                ),
            ));
        }

        write_buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut write_buffer = self
            .write_buffer
            .lock()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.description()))?;
        self.conn
            .send(write_buffer.as_slice())
            .map(|_| write_buffer.clear())
    }
}

impl std::io::Read for TUdpChannel {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.conn.recv(buf)
    }
}
