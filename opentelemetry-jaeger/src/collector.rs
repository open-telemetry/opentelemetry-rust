//! # HTTP Jaeger Collector Client
use crate::thrift::jaeger;
use crate::transport::THttpChannel;
use std::fmt;
use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
use thrift::transport::{ReadHalf, TIoChannel, WriteHalf};
use thrift::{protocol, TThriftClient};

/// `CollectorSyncClientHttp` implements the `TCollectorSyncClient` interface over HTTP
pub(crate) struct CollectorSyncClientHttp {
    client: jaeger::CollectorSyncClient<
        TBinaryInputProtocol<ReadHalf<THttpChannel>>,
        TBinaryOutputProtocol<WriteHalf<THttpChannel>>,
    >,
}

impl fmt::Debug for CollectorSyncClientHttp {
    /// Debug info
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CollectorSyncClientHttp")
            .field("client", &"CollectorSyncClient")
            .finish()
    }
}

impl CollectorSyncClientHttp {
    /// Create a new HTTP collector client
    pub(crate) fn new<T: Into<String>>(
        endpoint: T,
        username: Option<String>,
        password: Option<String>,
    ) -> thrift::Result<Self> {
        let transport = crate::transport::THttpChannel::new(endpoint, username, password)?;
        let (read, write) = transport.split()?;
        let client = jaeger::CollectorSyncClient::new(
            protocol::TBinaryInputProtocol::new(read, false),
            protocol::TBinaryOutputProtocol::new(write, true),
        );

        Ok(CollectorSyncClientHttp { client })
    }
}

impl jaeger::TCollectorSyncClient for CollectorSyncClientHttp {
    /// Submit list of Jaeger batches
    fn submit_batches(
        &mut self,
        batches: Vec<jaeger::Batch>,
    ) -> thrift::Result<Vec<jaeger::BatchSubmitResponse>> {
        // Ordinarily this would delegate to the client via `self.client.submit_batches(batches)`,
        // but the collector **does not actually accept a list of batches**. This is contrary to the
        // [definition of the service](https://github.com/jaegertracing/jaeger-idl/blob/d4063e359bc52eca57cdbeb92a179fd3aa0250d6/thrift/jaeger.thrift#L83-L85).
        // Instead, it accepts a single `jaeger::Batch` for consistency with tchannel intake.
        //
        // ¯\_(ツ)_/¯
        //
        // Details: https://github.com/jaegertracing/jaeger/blob/530e1f11508d7ee31307b9d70317cf6581faa5b7/cmd/collector/app/http_handler.go#L83-L88
        Ok(batches
            .into_iter()
            .map(|batch| {
                let proto = self.client.o_prot_mut();
                let ok = batch
                    .write_to_out_protocol(proto)
                    .and_then(|_| proto.flush())
                    .is_ok();

                jaeger::BatchSubmitResponse { ok }
            })
            .collect())
    }
}
