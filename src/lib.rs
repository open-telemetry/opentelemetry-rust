/*
   Copyright 2020 Vivint Smarthome

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use {
    derivative::Derivative,
    grpcio::{
        CallOption, ChannelBuilder, ChannelCredentials, Client, Environment, Marshaller, Method,
        MethodType,
    },
    opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter},
    protobuf::Message,
    std::{any::Any, sync::Arc},
};

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

/// Exports opentelemetry tracing spans to Google StackDriver.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StackDriverExporter {
    #[derivative(Debug = "ignore")]
    client: Client,
}

impl StackDriverExporter {
    pub fn new() -> Self {
        Self {
            client: Client::new(
                ChannelBuilder::new(Arc::new(Environment::new(num_cpus::get()))).secure_connect(
                    "cloudtrace.googleapis.com:443",
                    ChannelCredentials::google_default_credentials().unwrap(),
                ),
            ),
        }
    }
}

impl SpanExporter for StackDriverExporter {
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult {
        use proto::tracing::BatchWriteSpansRequest;
        use protobuf::well_known_types::Empty;
        let Self { client } = self;
        let req = BatchWriteSpansRequest::new();
        let result = client.unary_call(
            &Method {
                ty: MethodType::Unary,
                name: "google.devtools.cloudtrace.v2.TraceService.BatchWriteSpans",
                req_mar: Marshaller {
                    ser: |thiz: &BatchWriteSpansRequest, v: &mut Vec<u8>| {
                        thiz.write_to_vec(v).unwrap()
                    },
                    de: |b: &[u8]| {
                        let mut ret = BatchWriteSpansRequest::new();
                        ret.merge_from_bytes(b)?;
                        Ok(ret)
                    },
                },
                resp_mar: Marshaller {
                    ser: |thiz: &Empty, v: &mut Vec<u8>| thiz.write_to_vec(v).unwrap(),
                    de: |_: &[u8]| Ok(Empty::new()),
                },
            },
            &req,
            CallOption::default(),
        );
        match result {
            Ok(_) => ExportResult::Success,
            Err(e) => {
                log::error!("StackDriver push failed {:?}", e);
                ExportResult::FailedNotRetryable
            }
        }
    }

    fn shutdown(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        StackDriverExporter::new();
    }
}
