//! Additional Thrift transport implementations
#[cfg(feature = "collector_client")]
mod http;
mod udp;

#[cfg(feature = "collector_client")]
pub(crate) use http::THttpChannel;
pub(crate) use udp::TUdpChannel;
