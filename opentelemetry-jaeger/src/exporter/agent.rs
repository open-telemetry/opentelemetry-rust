//! # UDP Jaeger Agent Client
use crate::exporter::thrift::{
    agent::{self, TAgentSyncClient},
    jaeger,
};
use crate::exporter::transport::{TBufferChannel, TNoopChannel};
use std::fmt;
use std::net::{ToSocketAddrs, UdpSocket};
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
    conn: tokio::net::UdpSocket,
    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    conn: async_std::net::UdpSocket,
    buffer_client: BufferClient,
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
            conn: tokio::net::UdpSocket::from_std(conn)?,
            #[cfg(all(feature = "async-std", not(feature = "tokio")))]
            conn: async_std::net::UdpSocket::from(conn),
            buffer_client: BufferClient { buffer, client },
        })
    }

    /// Emit standard Jaeger batch
    pub(crate) async fn emit_batch(&mut self, batch: jaeger::Batch) -> thrift::Result<()> {
        // Write payload to buffer
        self.buffer_client.client.emit_batch(batch)?;
        let payload = self.buffer_client.buffer.take_bytes();

        // Write async to socket, reading from buffer
        write_to_socket(self, payload).await?;

        Ok(())
    }
}

#[cfg(all(not(feature = "async-std"), not(feature = "tokio")))]
async fn write_to_socket(client: &mut AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    client.conn.send(&payload)?;

    Ok(())
}

#[cfg(feature = "tokio")]
async fn write_to_socket(client: &mut AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    client.conn.send(&payload).await?;

    Ok(())
}

#[cfg(all(feature = "async-std", not(feature = "tokio")))]
async fn write_to_socket(client: &mut AgentAsyncClientUDP, payload: Vec<u8>) -> thrift::Result<()> {
    client.conn.send(&payload).await?;

    Ok(())
}
