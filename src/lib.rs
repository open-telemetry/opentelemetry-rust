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
    grpcio::{ChannelBuilder, ChannelCredentials, Client, Environment},
    opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter},
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
        ExportResult::Success
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
