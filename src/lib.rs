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
    grpcio::{ChannelBuilder, ChannelCredentials, Environment},
    opentelemetry::{
        api::core::Value,
        exporter::trace::{ExportResult, SpanData, SpanExporter},
    },
    protobuf::well_known_types::Timestamp,
    std::{
        any::Any,
        sync::{Arc, Mutex},
        time::SystemTime,
    },
    tokio::{prelude::Future, runtime::Runtime},
};

mod proto {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

use proto::{
    trace::{AttributeValue, Span_TimeEvent, Span_TimeEvent_Annotation, TruncatableString},
    tracing_grpc::TraceServiceClient,
};

/// Exports opentelemetry tracing spans to Google StackDriver.
///
/// As of the time of this writing, the opentelemetry crate exposes no link information
/// so this struct does not send link information.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StackDriverExporter {
    #[derivative(Debug = "ignore")]
    client: TraceServiceClient,
    project_name: String,
    runtime: Mutex<Runtime>,
}

impl StackDriverExporter {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            client: TraceServiceClient::new(
                ChannelBuilder::new(Arc::new(Environment::new(num_cpus::get()))).secure_connect(
                    "cloudtrace.googleapis.com:443",
                    ChannelCredentials::google_default_credentials().unwrap(),
                ),
            ),
            project_name: project_name.into(),
            runtime: Mutex::new(Runtime::new().unwrap()),
        }
    }
}

impl SpanExporter for StackDriverExporter {
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult {
        use proto::{trace::Span, tracing::BatchWriteSpansRequest};
        let Self {
            client,
            project_name,
            runtime,
        } = self;
        let mut req = BatchWriteSpansRequest::new();
        req.set_name(format!("projects/{}", project_name));
        for span in batch {
            let mut new_span = Span::new();
            new_span.set_name(format!(
                "projects/{}/traces/{}/spans/{}",
                project_name,
                hex::encode(span.context.trace_id().to_be_bytes()),
                hex::encode(span.context.span_id().to_be_bytes())
            ));
            new_span.set_display_name(to_truncate(span.name.clone()));
            new_span.set_span_id(hex::encode(span.context.span_id().to_be_bytes()));
            new_span.set_parent_span_id(hex::encode(span.parent_span_id.to_be_bytes()));
            new_span.set_start_time(system_time_to_timestamp(span.start_time));
            new_span.set_end_time(system_time_to_timestamp(span.end_time));
            new_span.mut_attributes().set_attribute_map(
                span.attributes
                    .iter()
                    .map(|kv| {
                        (
                            kv.key.inner().clone().into_owned(),
                            attribute_value_conversion(kv.value.clone()),
                        )
                    })
                    .collect(),
            );
            for event in span.message_events.iter() {
                new_span.mut_time_events().mut_time_event().push({
                    let mut time_event = Span_TimeEvent::new();
                    time_event.set_time(system_time_to_timestamp(event.timestamp));
                    time_event.set_annotation({
                        let mut a = Span_TimeEvent_Annotation::new();
                        a.set_description(to_truncate(event.message.clone()));
                        a
                    });
                    time_event
                });
            }
            req.mut_spans().push(new_span);
        }
        match client.batch_write_spans_async(&req) {
            Ok(f) => {
                runtime.lock().unwrap().spawn(
                    f.map(|_| ())
                        .map_err(|e| log::error!("StackDriver responded with error {:?}", e)),
                );
            }
            Err(e) => {
                log::error!("StackDriver push failed {:?}", e);
            }
        }
        ExportResult::Success
    }

    fn shutdown(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn system_time_to_timestamp(system: SystemTime) -> Timestamp {
    let d = system.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let mut t = Timestamp::new();
    t.set_seconds(d.as_secs() as i64);
    t.set_nanos(d.subsec_nanos() as i32);
    t
}

fn attribute_value_conversion(v: Value) -> AttributeValue {
    let mut a = AttributeValue::new();
    match v {
        Value::Bool(v) => a.set_bool_value(v),
        Value::Bytes(v) => a.set_string_value(to_truncate(hex::encode(&v))),
        Value::F64(v) => a.set_string_value(to_truncate(v.to_string())),
        Value::I64(v) => a.set_int_value(v),
        Value::String(v) => a.set_string_value(to_truncate(v)),
        Value::U64(v) => a.set_int_value(v as i64),
    }
    a
}

fn to_truncate(s: String) -> TruncatableString {
    let mut t = TruncatableString::new();
    t.set_value(s);
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        StackDriverExporter::new("fake-project");
    }
}
