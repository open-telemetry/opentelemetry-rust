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
        let conn = std::net::UdpSocket::bind(bind_addr(&host_port))?;
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
        let conn = std::net::UdpSocket::bind(bind_addr(&host_port))?;
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
        let conn = std::net::UdpSocket::bind(bind_addr(&host_port))?;
        conn.connect(host_port)?;
        Ok(async_std::net::UdpSocket::from(conn))
    }

    async fn write_to_socket(&self, socket: &Self::Socket, payload: Vec<u8>) -> thrift::Result<()> {
        socket.send(&payload).await?;

        Ok(())
    }
}

const INADDR_ANY: &str = "0.0.0.0:0";
const IN6ADDR_ANY: &str = "[::]:0";
/// Sample the first address provided to designate which IP family to bind the socket to.
/// Returns either INADDR_ANY or IN6ADDR_ANY
fn bind_addr<'a, T: ToSocketAddrs>(sockaddrs: &T) -> &'a str {
    let sockaddrs = sockaddrs.to_socket_addrs();
    if sockaddrs.is_err() {
        return INADDR_ANY;
    }
    sockaddrs.unwrap().next().map_or_else(
        || INADDR_ANY,
        |s| {
            if s.is_ipv4() {
                INADDR_ANY
            } else {
                IN6ADDR_ANY
            }
        },
    )
}
