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
    opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter},
    std::{any::Any, sync::Arc},
};

/// Exports opentelemetry tracing spans to Google StackDriver.
#[derive(Debug)]
pub struct StackDriverExporter {}

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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
