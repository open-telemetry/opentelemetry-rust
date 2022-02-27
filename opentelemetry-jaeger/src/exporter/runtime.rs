use async_trait::async_trait;
use opentelemetry::sdk::trace::TraceRuntime;
use std::net::ToSocketAddrs;

/// Jaeger Trace Runtime is an extension to [`TraceRuntime`].
///
/// [`TraceRuntime`]: opentelemetry::sdk::trace::TraceRuntime
#[async_trait]
pub trait JaegerTraceRuntime: TraceRuntime + std::fmt::Debug {
    /// A communication socket between Jaeger client and agent.
    type Socket: std::fmt::Debug + Send + Sync;

    /// Create a new communication socket.
    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket>;

    /// Send payload over the socket.
    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()>;
}

#[cfg(feature = "rt-tokio")]
#[async_trait]
impl JaegerTraceRuntime for opentelemetry::runtime::Tokio {
    type Socket = tokio::net::UdpSocket;

    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket> {
        let conn = std::net::UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;
        Ok(tokio::net::UdpSocket::from_std(conn)?)
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}

#[cfg(feature = "rt-tokio-current-thread")]
#[async_trait]
impl JaegerTraceRuntime for opentelemetry::runtime::TokioCurrentThread {
    type Socket = tokio::net::UdpSocket;

    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket> {
        let conn = std::net::UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;
        Ok(tokio::net::UdpSocket::from_std(conn)?)
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}

#[cfg(feature = "rt-async-std")]
#[async_trait]
impl JaegerTraceRuntime for opentelemetry::runtime::AsyncStd {
    type Socket = async_std::net::UdpSocket;

    fn create_socket<T: ToSocketAddrs>(&self, host_port: T) -> thrift::Result<Self::Socket> {
        let conn = std::net::UdpSocket::bind("0.0.0.0:0")?;
        conn.connect(host_port)?;
        Ok(async_std::net::UdpSocket::from(conn))
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}
