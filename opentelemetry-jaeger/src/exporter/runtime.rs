use opentelemetry::sdk::trace::TraceRuntime;
use async_trait::async_trait;
use std::net::{ToSocketAddrs, UdpSocket};

/// Jaeger Trace Runtime is an extension to [`TraceRuntime`]. Currently it provides a UDP socket used
/// by [`AgentAsyncClientUdp`].
/// 
/// [`TraceRuntime`]: opentelemetry::sdk::trace::TraceRuntime
/// [`AgentAsyncClientUdp`]: crate::exporter::agent::AgentAsyncClientUdp
#[async_trait]
pub trait JaegerTraceRuntime: TraceRuntime + std::fmt::Debug {
    type Socket: std::fmt::Debug + Send + Sync;
    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket>;
    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()>;
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))))]
#[async_trait]
impl JaegerTraceRuntime for opentelemetry::runtime::Tokio {
    type Socket = tokio::net::UdpSocket;

    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket> {
        let conn = UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;
        Ok(tokio::net::UdpSocket::from_std(conn)?)
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}

#[cfg(feature = "rt-async-std")]
#[cfg_attr(docrs, doc(cfg(feature = "rt-async-std")))]
#[async_trait]
impl JaegerTraceRuntime for opentelemetry::runtime::AsyncStd {
    type Socket = async_std::net::UdpSocket;

    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket> {
        let conn = UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;
        Ok(async_std::net::UdpSocket::from(conn))
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}
