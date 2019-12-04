//! # UDP Jaeger Agent Client
use crate::thrift::{agent, jaeger, zipkincore};
use crate::transport::TUdpChannel;
use std::fmt;
use std::net::ToSocketAddrs;
use thrift::protocol;
use thrift::protocol::{TCompactInputProtocol, TCompactOutputProtocol};
use thrift::transport::{ReadHalf, TIoChannel, WriteHalf};

/// `AgentSyncClientUDP` implements the `TAgentSyncClient` interface over UDP
pub(crate) struct AgentSyncClientUDP {
    client: agent::AgentSyncClient<
        TCompactInputProtocol<ReadHalf<TUdpChannel>>,
        TCompactOutputProtocol<WriteHalf<TUdpChannel>>,
    >,
}

impl fmt::Debug for AgentSyncClientUDP {
    /// Debug info
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AgentClientUDP")
            .field("client", &"AgentSyncClient")
            .finish()
    }
}

impl AgentSyncClientUDP {
    /// Create a new UDP agent client
    pub(crate) fn new<T: ToSocketAddrs>(
        host_port: T,
        max_packet_size: Option<usize>,
    ) -> thrift::Result<Self> {
        let transport = TUdpChannel::new(host_port, max_packet_size)?;
        let (read, write) = transport.split()?;
        let client = agent::AgentSyncClient::new(
            protocol::TCompactInputProtocol::new(read),
            protocol::TCompactOutputProtocol::new(write),
        );

        Ok(AgentSyncClientUDP { client })
    }
}

impl agent::TAgentSyncClient for AgentSyncClientUDP {
    /// Emit zipkin batch (Deprecated)
    fn emit_zipkin_batch(&mut self, spans: Vec<zipkincore::Span>) -> thrift::Result<()> {
        self.client.emit_zipkin_batch(spans)
    }

    /// Emit standard Jaeger batch
    fn emit_batch(&mut self, batch: jaeger::Batch) -> thrift::Result<()> {
        self.client.emit_batch(batch)
    }
}
