use crate::trace::exporter::datadog::intern::StringInterner;
use crate::trace::exporter::datadog::Error;
use opentelemetry::sdk::export::trace;
use opentelemetry::{Key, Value};
use std::time::SystemTime;

// Protocol documentation sourced from https://github.com/DataDog/datadog-agent/blob/c076ea9a1ffbde4c76d35343dbc32aecbbf99cb9/pkg/trace/api/version.go
//
// The payload is an array containing exactly 2 elements:
//
// 	1. An array of all unique strings present in the payload (a dictionary referred to by index).
// 	2. An array of traces, where each trace is an array of spans. A span is encoded as an array having
// 	   exactly 12 elements, representing all span properties, in this exact order:
//
// 		 0: Service   (uint32)
// 		 1: Name      (uint32)
// 		 2: Resource  (uint32)
// 		 3: TraceID   (uint64)
// 		 4: SpanID    (uint64)
// 		 5: ParentID  (uint64)
// 		 6: Start     (int64)
// 		 7: Duration  (int64)
// 		 8: Error     (int32)
// 		 9: Meta      (map[uint32]uint32)
// 		10: Metrics   (map[uint32]float64)
// 		11: Type      (uint32)
//
// 	Considerations:
//
// 	- The "uint32" typed values in "Service", "Name", "Resource", "Type", "Meta" and "Metrics" represent
// 	  the index at which the corresponding string is found in the dictionary. If any of the values are the
// 	  empty string, then the empty string must be added into the dictionary.
//
// 	- None of the elements can be nil. If any of them are unset, they should be given their "zero-value". Here
// 	  is an example of a span with all unset values:
//
// 		 0: 0                    // Service is "" (index 0 in dictionary)
// 		 1: 0                    // Name is ""
// 		 2: 0                    // Resource is ""
// 		 3: 0                    // TraceID
// 		 4: 0                    // SpanID
// 		 5: 0                    // ParentID
// 		 6: 0                    // Start
// 		 7: 0                    // Duration
// 		 8: 0                    // Error
// 		 9: map[uint32]uint32{}  // Meta (empty map)
// 		10: map[uint32]float64{} // Metrics (empty map)
// 		11: 0                    // Type is ""
//
// 		The dictionary in this case would be []string{""}, having only the empty string at index 0.
//
pub(crate) fn encode(service_name: &str, traces: Vec<Vec<trace::SpanData>>) -> Result<Vec<u8>, Error> {
    let mut interner = StringInterner::new();
    let mut encoded_traces = encode_traces(&mut interner, service_name, traces)?;

    let mut payload = Vec::new();
    rmp::encode::write_array_len(&mut payload, 2)?;

    rmp::encode::write_array_len(&mut payload, interner.len() as u32)?;
    for data in interner.iter() {
        rmp::encode::write_str(&mut payload, data)?;
    }

    payload.append(&mut encoded_traces);

    Ok(payload)
}

fn encode_traces(
    interner: &mut StringInterner,
    service_name: &str,
    traces: Vec<Vec<trace::SpanData>>,
) -> Result<Vec<u8>, Error> {
    let mut encoded = Vec::new();
    rmp::encode::write_array_len(&mut encoded, traces.len() as u32)?;

    let service_interned = interner.intern(&service_name);

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

            let span_type = match span.attributes.get(&Key::new("span.type")) {
                Some(Value::String(s)) => interner.intern(s.as_ref()),
                _ => interner.intern(""),
            };

            // Datadog span name is OpenTelemetry component name - see module docs for more information
            rmp::encode::write_array_len(&mut encoded, 12)?;
            rmp::encode::write_u32(&mut encoded, service_interned)?;
            rmp::encode::write_u32(&mut encoded, interner.intern(span.instrumentation_lib.name))?;
            rmp::encode::write_u32(&mut encoded, interner.intern(&span.name))?;
            rmp::encode::write_u64(&mut encoded, span.span_context.trace_id().to_u128() as u64)?;
            rmp::encode::write_u64(&mut encoded, span.span_context.span_id().to_u64())?;
            rmp::encode::write_u64(&mut encoded, span.parent_span_id.to_u64())?;
            rmp::encode::write_i64(&mut encoded, start)?;
            rmp::encode::write_i64(&mut encoded, duration)?;
            rmp::encode::write_i32(&mut encoded, span.status_code as i32)?;
            rmp::encode::write_map_len(&mut encoded, span.attributes.len() as u32)?;
            for (key, value) in span.attributes.iter() {
                rmp::encode::write_u32(&mut encoded, interner.intern(key.as_str()))?;
                rmp::encode::write_u32(&mut encoded, interner.intern(value.as_str().as_ref()))?;
            }
            rmp::encode::write_map_len(&mut encoded, 0)?;
            rmp::encode::write_u32(&mut encoded, span_type)?;
        }
    }

    Ok(encoded)
}
