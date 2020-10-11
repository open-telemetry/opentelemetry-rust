//! # UDP Jaeger Agent Client
use crate::thrift::{
    agent::{self, TAgentSyncClient},
    jaeger,
};
use crate::transport::{TBufferChannel, TNoopChannel};
use std::fmt;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::Mutex;
use thrift::{
    protocol::{TCompactInputProtocol, TCompactOutputProtocol},
    transport::{ReadHalf, TIoChannel, WriteHalf},
};

struct BufferClient {
    buffer: ReadHalf<TBufferChannel>,
    client: agent::AgentSyncClient<
        TCompactInputProtocol<TNoopChannel>,
        TCompactOutputProtocol<WriteHalf<TBufferChannel>>,
    >,
}

impl fmt::Debug for BufferClient {
    /// Debug info
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufferClient")
            .field("buffer", &self.buffer)
            .field("client", &"AgentSyncClient")
            .finish()
    }
}

/// `AgentAsyncClientUDP` implements an async version of the `TAgentSyncClient`
/// interface over UDP.
#[derive(Debug)]
pub(crate) struct AgentAsyncClientUDP {
    #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
    conn: UdpSocket,
    #[cfg(feature = "tokio")]
    conn: tokio::sync::Mutex<tokio::net::UdpSocket>,
    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    conn: async_std::sync::Mutex<async_std::net::UdpSocket>,
    buffer_client: Mutex<BufferClient>,
}

impl AgentAsyncClientUDP {
    /// Create a new UDP agent client
    pub(crate) fn new<T: ToSocketAddrs>(host_port: T) -> thrift::Result<Self> {
        let (buffer, write) = TBufferChannel::with_capacity(512).split()?;
        let client = agent::AgentSyncClient::new(
            TCompactInputProtocol::new(TNoopChannel),
            TCompactOutputProtocol::new(write),
        );

        let conn = UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;

        Ok(AgentAsyncClientUDP {
            #[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
            conn,
            #[cfg(feature = "tokio")]
            conn: tokio::sync::Mutex::new(tokio::net::UdpSocket::from_std(conn)?),
            #[cfg(all(feature = "async-std", not(feature = "tokio")))]
            conn: async_std::sync::Mutex::new(async_std::net::UdpSocket::from(conn)),
            buffer_client: Mutex::new(BufferClient { buffer, client }),
        })
    }

    /// Emit standard Jaeger batch
    pub(crate) async fn emit_batch(&self, batch: jaeger::Batch) -> thrift::Result<()> {
        // Write payload to buffer
        let payload = self
            .buffer_client
            .lock()
            .map_err(|err| {
                thrift::Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    err.to_string(),
                ))
            })
            .and_then(|mut buffer_client| {
                // Write to tmp buffer
                buffer_client.client.emit_batch(batch)?;
                // extract written payload, clearing buffer
                let payload = buffer_client.buffer.take_bytes();

                Ok(payload)
            })?;

        // Write async to socket, reading from buffer
        write_to_socket(self, payload).await?;

        Ok(())
    }
}

#[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
async fn write_to_socket(client: &AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    client.conn.send(&payload)?;

    Ok(())
}

#[cfg(feature = "tokio")]
async fn write_to_socket(client: &AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    let mut conn = client.conn.lock().await;
    conn.send(&payload).await?;

    Ok(())
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
async fn write_to_socket(client: &AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    let conn = client.conn.lock().await;
    conn.send(&payload).await?;

    Ok(())
}
