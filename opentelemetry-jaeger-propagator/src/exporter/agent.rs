//! # UDP Jaeger Agent Client
use crate::exporter::address_family;
use crate::exporter::runtime::JaegerTraceRuntime;
use crate::exporter::thrift::{
    agent::{self, TAgentSyncClient},
    jaeger,
};
use crate::exporter::transport::{TBufferChannel, TNoopChannel};
use std::fmt;
use std::net::{SocketAddr, UdpSocket};
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

/// `AgentSyncClientUDP` implements a version of the `TAgentSyncClient`
/// interface over UDP.
#[derive(Debug)]
pub(crate) struct AgentSyncClientUdp {
    conn: UdpSocket,
    buffer_client: BufferClient,
    max_packet_size: usize,
    auto_split: bool,
}

impl AgentSyncClientUdp {
    /// Create a new UDP agent client
    pub(crate) fn new(
        max_packet_size: usize,
        auto_split: bool,
        agent_address: Vec<SocketAddr>,
    ) -> thrift::Result<Self> {
        let (buffer, write) = TBufferChannel::with_capacity(max_packet_size).split()?;
        let client = agent::AgentSyncClient::new(
            TCompactInputProtocol::new(TNoopChannel),
            TCompactOutputProtocol::new(write),
        );

        let conn = UdpSocket::bind(address_family(agent_address.as_slice()))?;
        conn.connect(agent_address.as_slice())?;

        Ok(AgentSyncClientUdp {
            conn,
            buffer_client: BufferClient { buffer, client },
            max_packet_size,
            auto_split,
        })
    }

    /// Emit standard Jaeger batch
    pub(crate) fn emit_batch(&mut self, batch: jaeger::Batch) -> thrift::Result<()> {
        if !self.auto_split {
            let payload = serialize_batch(&mut self.buffer_client, batch, self.max_packet_size)?;
            self.conn.send(&payload)?;
            return Ok(());
        }

        let mut buffers = vec![];
        serialize_batch_vectored(
            &mut self.buffer_client,
            batch,
            self.max_packet_size,
            &mut buffers,
        )?;
        for payload in buffers {
            self.conn.send(&payload)?;
        }

        Ok(())
    }
}

/// `AgentAsyncClientUDP` implements an async version of the `TAgentSyncClient`
/// interface over UDP.
#[derive(Debug)]
pub(crate) struct AgentAsyncClientUdp<R: JaegerTraceRuntime> {
    runtime: R,
    conn: <R as JaegerTraceRuntime>::Socket,
    buffer_client: BufferClient,
    max_packet_size: usize,
    auto_split: bool,
}

impl<R: JaegerTraceRuntime> AgentAsyncClientUdp<R> {
    /// Create a new UDP agent client
    pub(crate) fn new(
        max_packet_size: usize,
        runtime: R,
        auto_split: bool,
        agent_address: Vec<SocketAddr>,
    ) -> thrift::Result<Self> {
        let (buffer, write) = TBufferChannel::with_capacity(max_packet_size).split()?;
        let client = agent::AgentSyncClient::new(
            TCompactInputProtocol::new(TNoopChannel),
            TCompactOutputProtocol::new(write),
        );

        let conn = runtime.create_socket(agent_address.as_slice())?;

        Ok(AgentAsyncClientUdp {
            runtime,
            conn,
            buffer_client: BufferClient { buffer, client },
            max_packet_size,
            auto_split,
        })
    }

    /// Emit standard Jaeger batch
    pub(crate) async fn emit_batch(&mut self, batch: jaeger::Batch) -> thrift::Result<()> {
        if !self.auto_split {
            let payload = serialize_batch(&mut self.buffer_client, batch, self.max_packet_size)?;
            self.runtime.write_to_socket(&self.conn, payload).await?;
            return Ok(());
        }

        let mut buffers = vec![];
        serialize_batch_vectored(
            &mut self.buffer_client,
            batch,
            self.max_packet_size,
            &mut buffers,
        )?;
        for payload in buffers {
            self.runtime.write_to_socket(&self.conn, payload).await?;
        }

        Ok(())
    }
}

fn serialize_batch(
    client: &mut BufferClient,
    batch: jaeger::Batch,
    max_packet_size: usize,
) -> thrift::Result<Vec<u8>> {
    client.client.emit_batch(batch)?;
    let payload = client.buffer.take_bytes();

    if payload.len() > max_packet_size {
        return Err(thrift::ProtocolError::new(
                thrift::ProtocolErrorKind::SizeLimit,
                format!(
                    "jaeger exporter payload size of {} bytes over max UDP packet size of {} bytes. Try setting a smaller batch size or turn auto split on.",
                    payload.len(),
                    max_packet_size,
                ),
            )
                .into());
    }

    Ok(payload)
}

fn serialize_batch_vectored(
    client: &mut BufferClient,
    mut batch: jaeger::Batch,
    max_packet_size: usize,
    output: &mut Vec<Vec<u8>>,
) -> thrift::Result<()> {
    client.client.emit_batch(batch.clone())?;
    let payload = client.buffer.take_bytes();

    if payload.len() <= max_packet_size {
        output.push(payload);
        return Ok(());
    }

    if batch.spans.len() <= 1 {
        return Err(thrift::ProtocolError::new(
            thrift::ProtocolErrorKind::SizeLimit,
            format!(
                "single span's jaeger exporter payload size of {} bytes over max UDP packet size of {} bytes",
                payload.len(),
                max_packet_size,
            ),
        )
            .into());
    }

    let mid = batch.spans.len() / 2;
    let new_spans = batch.spans.drain(mid..).collect::<Vec<_>>();
    let new_batch = jaeger::Batch::new(batch.process.clone(), new_spans);

    serialize_batch_vectored(client, batch, max_packet_size, output)?;
    serialize_batch_vectored(client, new_batch, max_packet_size, output)?;

    Ok(())
}
