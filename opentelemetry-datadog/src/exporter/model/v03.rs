use crate::exporter::model::{Error, SAMPLING_PRIORITY_KEY};
use crate::exporter::ModelConfig;
use opentelemetry::trace::Status;
use opentelemetry_sdk::export::trace::SpanData;
use std::time::SystemTime;

pub(crate) fn encode<S, N, R>(
    model_config: &ModelConfig,
    traces: Vec<Vec<SpanData>>,
    get_service_name: S,
    get_name: N,
    get_resource: R,
) -> Result<Vec<u8>, Error>
where
    for<'a> S: Fn(&'a SpanData, &'a ModelConfig) -> &'a str,
    for<'a> N: Fn(&'a SpanData, &'a ModelConfig) -> &'a str,
    for<'a> R: Fn(&'a SpanData, &'a ModelConfig) -> &'a str,
{
    let mut encoded = Vec::new();
    rmp::encode::write_array_len(&mut encoded, traces.len() as u32)?;

    for trace in traces.into_iter() {
        rmp::encode::write_array_len(&mut encoded, trace.len() as u32)?;

        for span in trace.into_iter() {
            // Safe until the year 2262 when Datadog will need to change their API
            let start = span
                .start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64;

            let duration = span
                .end_time
                .duration_since(span.start_time)
                .map(|x| x.as_nanos() as i64)
                .unwrap_or(0);

            // if let Some(Value::String(s)) = span.attributes.get(&Key::new("span.type")) {
            //     rmp::encode::write_map_len(&mut encoded, 12)?;
            //     rmp::encode::write_str(&mut encoded, "type")?;
            //     rmp::encode::write_str(&mut encoded, s.as_str())?;
            // } else {
            //     rmp::encode::write_map_len(&mut encoded, 11)?;
            // }

            let mut span_type_found = false;
            for kv in &span.attributes {
                if kv.key.as_str() == "span.type" {
                    span_type_found = true;
                    rmp::encode::write_map_len(&mut encoded, 12)?;
                    rmp::encode::write_str(&mut encoded, "type")?;
                    rmp::encode::write_str(&mut encoded, kv.value.as_str().as_ref())?;
                    break;
                }
            }

            if !span_type_found {
                rmp::encode::write_map_len(&mut encoded, 11)?;
            }

            // Datadog span name is OpenTelemetry component name - see module docs for more information
            rmp::encode::write_str(&mut encoded, "service")?;
            rmp::encode::write_str(&mut encoded, get_service_name(&span, model_config))?;

            rmp::encode::write_str(&mut encoded, "name")?;
            rmp::encode::write_str(&mut encoded, get_name(&span, model_config))?;

            rmp::encode::write_str(&mut encoded, "resource")?;
            rmp::encode::write_str(&mut encoded, get_resource(&span, model_config))?;

            rmp::encode::write_str(&mut encoded, "trace_id")?;
            rmp::encode::write_u64(
                &mut encoded,
                u128::from_be_bytes(span.span_context.trace_id().to_bytes()) as u64,
            )?;

            rmp::encode::write_str(&mut encoded, "span_id")?;
            rmp::encode::write_u64(
                &mut encoded,
                u64::from_be_bytes(span.span_context.span_id().to_bytes()),
            )?;

            rmp::encode::write_str(&mut encoded, "parent_id")?;
            rmp::encode::write_u64(
                &mut encoded,
                u64::from_be_bytes(span.parent_span_id.to_bytes()),
            )?;

            rmp::encode::write_str(&mut encoded, "start")?;
            rmp::encode::write_i64(&mut encoded, start)?;

            rmp::encode::write_str(&mut encoded, "duration")?;
            rmp::encode::write_i64(&mut encoded, duration)?;

            rmp::encode::write_str(&mut encoded, "error")?;
            rmp::encode::write_i32(
                &mut encoded,
                match span.status {
                    Status::Error { .. } => 1,
                    _ => 0,
                },
            )?;

            rmp::encode::write_str(&mut encoded, "meta")?;
            rmp::encode::write_map_len(
                &mut encoded,
                (span.attributes.len() + span.resource.len()) as u32,
            )?;
            for (key, value) in span.resource.iter() {
                rmp::encode::write_str(&mut encoded, key.as_str())?;
                rmp::encode::write_str(&mut encoded, value.as_str().as_ref())?;
            }
            for kv in span.attributes.iter() {
                rmp::encode::write_str(&mut encoded, kv.key.as_str())?;
                rmp::encode::write_str(&mut encoded, kv.value.as_str().as_ref())?;
            }

            rmp::encode::write_str(&mut encoded, "metrics")?;
            rmp::encode::write_map_len(&mut encoded, 1)?;
            rmp::encode::write_str(&mut encoded, SAMPLING_PRIORITY_KEY)?;
            rmp::encode::write_f64(
                &mut encoded,
                if span.span_context.is_sampled() {
                    1.0
                } else {
                    0.0
                },
            )?;
        }
    }

    Ok(encoded)
}
