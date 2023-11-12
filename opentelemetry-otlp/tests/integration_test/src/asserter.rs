use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
use std::fs::File;

// assert two resource spans is the same except for the timestamps in it
pub fn assert_span_eq(left: &ResourceSpans, right: &ResourceSpans) {
    assert_eq!(left.resource, right.resource);
    assert_eq!(left.schema_url, right.schema_url);

    assert_eq!(left.scope_spans.len(), right.scope_spans.len());
    let length = left.scope_spans.len();
    for i in 0..length {
        let left_spans = &left.scope_spans[i];
        let right_spans = &right.scope_spans[i];
        assert_eq!(left_spans.scope, right_spans.scope);
        assert_eq!(left_spans.schema_url, right_spans.schema_url);

        assert_eq!(left_spans.spans.len(), right_spans.spans.len());
        let length = left_spans.spans.len();
        for i in 0..length {
            let left_span = &left_spans.spans[i];
            let right_span = &right_spans.spans[i];
            assert_eq!(left_span.name, right_span.name);
            assert_eq!(left_span.kind, right_span.kind);
            // ignore start_time_unit_nano
            // ignore end_time_unit_nano
            assert_eq!(left_span.attributes, right_span.attributes);
            assert_eq!(left_span.links, right_span.links);
            assert_eq!(left_span.status, right_span.status);

            assert_eq!(left_span.events.len(), right_span.events.len());
            let length = left_span.events.len();
            for i in 0..length {
                let left_event = &left_span.events[i];
                let right_event = &right_span.events[i];
                assert_eq!(left_event.name, right_event.name);
                // ignore time_unix_nano
                assert_eq!(left_event.attributes, right_event.attributes);
                assert_eq!(
                    left_event.dropped_attributes_count,
                    right_event.dropped_attributes_count
                );
            }
        }
    }
}

pub fn read_spans_from_json(file: File) -> ResourceSpans {
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
