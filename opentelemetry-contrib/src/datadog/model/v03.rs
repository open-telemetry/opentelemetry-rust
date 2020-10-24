use crate::datadog::model::Error;
use opentelemetry::api::{Key, Value};
use opentelemetry::exporter::trace;
use std::time::SystemTime;

pub(crate) fn encode(service_name: &str, spans: Vec<trace::SpanData>) -> Result<Vec<u8>, Error> {
    let mut encoded = Vec::new();
    rmp::encode::write_array_len(&mut encoded, spans.len() as u32)?;

    for span in spans.into_iter() {
        // API supports but doesn't mandate grouping spans with the same trace ID
        rmp::encode::write_array_len(&mut encoded, 1)?;

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

        if let Some(Value::String(s)) = span.attributes.get(&Key::new("span.type")) {
            rmp::encode::write_map_len(&mut encoded, 11)?;
            rmp::encode::write_str(&mut encoded, "type")?;
            rmp::encode::write_str(&mut encoded, s.as_str())?;
        } else {
            rmp::encode::write_map_len(&mut encoded, 10)?;
        }

        // Datadog span name is OpenTelemetry component name - see module docs for more information
        rmp::encode::write_str(&mut encoded, "service")?;
        rmp::encode::write_str(&mut encoded, service_name)?;

        rmp::encode::write_str(&mut encoded, "name")?;
        rmp::encode::write_str(&mut encoded, span.instrumentation_lib.name)?;

        rmp::encode::write_str(&mut encoded, "resource")?;
        rmp::encode::write_str(&mut encoded, &span.name)?;

        rmp::encode::write_str(&mut encoded, "trace_id")?;
        rmp::encode::write_u64(
            &mut encoded,
            span.span_context.trace_id().to_u128() as u64,
        )?;

        rmp::encode::write_str(&mut encoded, "span_id")?;
        rmp::encode::write_u64(&mut encoded, span.span_context.span_id().to_u64())?;

        rmp::encode::write_str(&mut encoded, "parent_id")?;
        rmp::encode::write_u64(&mut encoded, span.parent_span_id.to_u64())?;

        rmp::encode::write_str(&mut encoded, "start")?;
        rmp::encode::write_i64(&mut encoded, start)?;

        rmp::encode::write_str(&mut encoded, "duration")?;
        rmp::encode::write_i64(&mut encoded, duration)?;

        rmp::encode::write_str(&mut encoded, "error")?;
        rmp::encode::write_i32(&mut encoded, span.status_code as i32)?;

        rmp::encode::write_str(&mut encoded, "meta")?;
        rmp::encode::write_map_len(&mut encoded, span.attributes.len() as u32)?;
        for (key, value) in span.attributes.iter() {
            let value_string: String = value.into();
            rmp::encode::write_str(&mut encoded, key.as_str())?;
            rmp::encode::write_str(&mut encoded, value_string.as_str())?;
        }
    }

    Ok(encoded)
}
